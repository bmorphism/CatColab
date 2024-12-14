/*! Models of double theories.

A model of a double theory is a category (or categories) equipped with
operations specified by the theory, categorifying the familiar idea from logic
that a model of a theory is a set (or sets) equipped with operations. For
background on double theories, see the [`theory`](super::theory) module.

In the case of a *simple* double theory, which amounts to a small double
category, a **model** of the theory is a span-valued *lax* double functor out of
the theory. Such a model is a "lax copresheaf," categorifying the notion of a
copresheaf or set-valued functor. Though they are "just" lax double functors,
models are a [concept with an
attitude](https://ncatlab.org/nlab/show/concept+with+an+attitude). To bring out
the intended intuition we introduce new jargon, building on that for double
theories.

# Terminology

A model of a double theory consists of elements of two kinds:

1. **Objects**, each assigned an object type in the theory;

2. **Morphisms**, each having a domain and a codomain object and assigned a
   morphism type in the theory, compatibly with the domain and codomain types;

In addition, a model has the following operations:

- **Object action**: object operations in the theory act on objects in the model
  to produce new objects;

- **Morphism action**: morphism operations in the theory act on morphisms in
  the model to produce new morphisms, compatibly with the object action;

- **Composition**: a path of morphisms in the model has a composite morphism,
  whose type is the composite of the corresponding morphism types.
 */

use std::hash::{BuildHasher, BuildHasherDefault, Hash, RandomState};
use std::iter::Iterator;
use std::sync::Arc;

use derivative::Derivative;
use ustr::{IdentityHasher, Ustr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde-wasm")]
use tsify_next::Tsify;

use super::theory::{DblTheory, DiscreteDblTheory};
use crate::one::fin_category::{FpCategory, InvalidFpCategory, UstrFinCategory};
use crate::one::*;
use crate::validate::{self, Validate};
use crate::zero::{Column, FinSet, HashColumn, HashFinSet, IndexedHashColumn, Mapping, Set};

use super::theory::*;

/** A model of a double theory.

As always in logic, a model makes sense only relative to a theory, but a theory
can have many different models. So, in Rust, a model needs access to its theory
but should not *own* its theory. Implementors of this trait might use an
immutable shared reference to the theory.

Objects and morphisms in a model are typed by object types and morphism types in
the theory. There is a design choice about whether identifiers for objects
([`Ob`](Category::Ob)) and morphisms ([`Mor`](Category::Mor)) are unique
relative to their types or globally within the model. If we took the first
approach (as we do in the Julia package
[ACSets.jl](https://github.com/AlgebraicJulia/ACSets.jl)), one could only make
sense of objects and morphisms when their types are known, so the early methods
in the trait would look like this:

```ignore
fn has_ob(&self, x: &Self::Ob, t: &Self::ObType) -> bool;
fn has_mor(&self, m: &Self::Mor, t: &Self::MorType) -> bool;
fn dom(&self, m: &Self::Mor, t: &Self::MorType) -> Self::Ob;
fn cod(&self, m: &Self::Mor, t: &Self::MorType) -> Self::Ob;
```

It will be more convenient for us to take the second approach since in our usage
object and morphism identifiers will be globally unique in a very strong sense
(something like UUIDs).
 */
pub trait DblModel: Category {
    /// Rust type of object types defined in the theory.
    type ObType: Eq;

    /// Rust type of morphism types defined in the theory.
    type MorType: Eq;

    /// Type of operations on objects defined in the theory.
    type ObOp: Eq;

    /// Type of operations on morphisms defined in the theory.
    type MorOp: Eq;

    /// The type of double theory that this is a model of.
    type Theory: DblTheory<
        ObType = Self::ObType,
        MorType = Self::MorType,
        ObOp = Self::ObOp,
        MorOp = Self::MorOp,
    >;

    /// The underlying theory that this is a model of
    fn theory(&self) -> &Self::Theory;

    /// Type of object.
    fn ob_type(&self, x: &Self::Ob) -> Self::ObType;

    /// Type of morphism.
    fn mor_type(&self, m: &Self::Mor) -> Self::MorType;

    /// Acts on an object with an object operation.
    fn ob_act(&self, x: Self::Ob, f: &Self::ObOp) -> Self::Ob;

    /// Acts on a morphism with a morphism operation.
    fn mor_act(&self, m: Self::Mor, α: &Self::MorOp) -> Self::Mor;
}

/// A finitely-generated double model
pub trait FgDblModel: DblModel + FgCategory {
    /// Type of an object generator.
    fn ob_gen_type(&self, ob: &Self::ObGen) -> Self::ObType;

    /// Type of a morphism generator.
    fn mor_gen_type(&self, mor: &Self::MorGen) -> Self::MorType;

    /// Iterates over object generators in the model of a given object type.
    fn object_generators_with_type(
        &self,
        obtype: &Self::ObType,
    ) -> impl Iterator<Item = Self::ObGen> {
        self.object_generators().filter(move |ob| self.ob_gen_type(ob) == *obtype)
    }

    /// Iterates over morphism generators in the model of a given morphism type.
    fn morphism_generators_with_type(
        &self,
        mortype: &Self::MorType,
    ) -> impl Iterator<Item = Self::MorGen> {
        self.morphism_generators().filter(move |mor| self.mor_gen_type(mor) == *mortype)
    }
}

/** A finitely presented model of a discrete double theory.

Since discrete double theory has only identity operations, such a model is a
finite presentation of a category sliced over the object and morphism types
comprising the theory. A type theorist would call it a ["displayed
category"](https://ncatlab.org/nlab/show/displayed+category).
*/
#[derive(Clone, Derivative, Debug)]
#[derivative(PartialEq(bound = "Id: Eq + Hash"))]
#[derivative(Eq(bound = "Id: Eq + Hash"))]
pub struct DiscreteDblModel<Id, Cat: FgCategory> {
    #[derivative(PartialEq(compare_with = "Arc::ptr_eq"))]
    theory: Arc<DiscreteDblTheory<Cat>>,
    category: FpCategory<Id, Id, Id>,
    ob_types: IndexedHashColumn<Id, Cat::Ob>,
    mor_types: IndexedHashColumn<Id, Cat::Mor>,
}

/// A model of a discrete double theory where both theoy and model have keys of
/// type `Ustr`.
pub type UstrDiscreteDblModel = DiscreteDblModel<Ustr, UstrFinCategory>;
// NOTE: We are leaving a small optimization on the table by not using the
// `IdentityHasher` but adding that extra type parameter quickly gets annoying
// because it has to be propagated everywhere, including into model morphisms.

impl<Id, Cat> DiscreteDblModel<Id, Cat>
where
    Id: Eq + Clone + Hash,
    Cat: FgCategory,
    Cat::Ob: Hash,
    Cat::Mor: Hash,
{
    /// Creates an empty model of the given theory.
    pub fn new(theory: Arc<DiscreteDblTheory<Cat>>) -> Self {
        Self {
            theory,
            category: Default::default(),
            ob_types: Default::default(),
            mor_types: Default::default(),
        }
    }

    /// Returns a reference-counting pointer to the theory for this model.
    pub fn theory_arc(&self) -> Arc<DiscreteDblTheory<Cat>> {
        self.theory.clone()
    }

    /// Returns the underlying graph of the model.
    pub fn generating_graph(&self) -> &impl FinGraph<V = Id, E = Id> {
        self.category.generators()
    }

    /// Is the model freely generated?
    pub fn is_free(&self) -> bool {
        self.category.is_free()
    }

    /// Adds a basic object to the model.
    pub fn add_ob(&mut self, x: Id, typ: Cat::Ob) -> bool {
        self.ob_types.set(x.clone(), typ);
        self.category.add_ob_generator(x)
    }

    /// Adds a basic morphism to the model.
    pub fn add_mor(&mut self, f: Id, dom: Id, cod: Id, typ: Cat::Mor) -> bool {
        self.mor_types.set(f.clone(), typ);
        self.category.add_mor_generator(f, dom, cod)
    }

    /// Adds an equation to the model, making it not free.
    pub fn add_equation(&mut self, key: Id, eq: PathEq<Id, Id>) {
        self.category.add_equation(key, eq);
    }

    /// Adds a basic morphism to the model without setting its (co)domain.
    pub fn make_mor(&mut self, f: Id, typ: Cat::Mor) -> bool {
        self.mor_types.set(f.clone(), typ);
        self.category.make_mor_generator(f)
    }

    /// Gets the domain of a basic morphism, if it is set.
    pub fn get_dom(&self, f: &Id) -> Option<&Id> {
        self.category.get_dom(f)
    }

    /// Gets the codomain of a basic morphism, if it is set.
    pub fn get_cod(&self, f: &Id) -> Option<&Id> {
        self.category.get_cod(f)
    }

    /// Sets the domain of a basic morphism.
    pub fn set_dom(&mut self, f: Id, x: Id) -> Option<Id> {
        self.category.set_dom(f, x)
    }

    /// Sets the codomain of a basic morphism.
    pub fn set_cod(&mut self, f: Id, x: Id) -> Option<Id> {
        self.category.set_cod(f, x)
    }

    /// Iterates over failures to be well-defined model.
    pub fn iter_invalid(&self) -> impl Iterator<Item = InvalidDiscreteDblModel<Id>> + '_ {
        type Invalid<Id> = InvalidDiscreteDblModel<Id>;
        let category_errors = self.category.iter_invalid().map(|err| match err {
            InvalidFpCategory::Dom(e) => Invalid::Dom(e),
            InvalidFpCategory::Cod(e) => Invalid::Cod(e),
            InvalidFpCategory::EqLhs(eq) => Invalid::EqLhs(eq),
            InvalidFpCategory::EqRhs(eq) => Invalid::EqRhs(eq),
            InvalidFpCategory::EqSrc(eq) => Invalid::EqSrc(eq),
            InvalidFpCategory::EqTgt(eq) => Invalid::EqTgt(eq),
        });
        let ob_type_errors = self.category.object_generators().filter_map(|x| {
            if self.theory.has_ob_type(&self.ob_type(&x)) {
                None
            } else {
                Some(Invalid::ObType(x))
            }
        });
        let mor_type_errors = self.category.morphism_generators().flat_map(|e| {
            let mut errs = Vec::new();
            let mor_type = self.mor_gen_type(&e);
            if self.theory.has_mor_type(&mor_type) {
                if self.category.get_dom(&e).map_or(false, |x| {
                    self.has_ob(x) && self.ob_type(x) != self.theory.src(&mor_type)
                }) {
                    errs.push(Invalid::DomType(e.clone()));
                }
                if self.category.get_cod(&e).map_or(false, |x| {
                    self.has_ob(x) && self.ob_type(x) != self.theory.tgt(&mor_type)
                }) {
                    errs.push(Invalid::CodType(e));
                }
            } else {
                errs.push(Invalid::MorType(e));
            }
            errs.into_iter()
        });
        category_errors.chain(ob_type_errors).chain(mor_type_errors)
    }

    /** Infer missing data in the model, where possible.

    Objects used in the domain or codomain of morphisms, but not contained as
    objects of the model, are added and their types are inferred. It is not
    always possible to do this consistently, so it is important to `validate`
    the model even after calling this method.
    */
    pub fn infer_missing(&mut self) {
        let edges: Vec<_> = self.morphism_generators().collect();
        for e in edges {
            if let Some(x) = self.get_dom(&e).filter(|x| !self.has_ob(x)) {
                let ob_type = self.theory.src(&self.mor_gen_type(&e));
                self.add_ob(x.clone(), ob_type);
            }
            if let Some(x) = self.get_cod(&e).filter(|x| !self.has_ob(x)) {
                let ob_type = self.theory.tgt(&self.mor_gen_type(&e));
                self.add_ob(x.clone(), ob_type);
            }
        }
    }
}

impl<Id, Cat> Category for DiscreteDblModel<Id, Cat>
where
    Id: Eq + Clone + Hash,
    Cat: FgCategory,
    Cat::Ob: Hash,
    Cat::Mor: Hash,
{
    type Ob = Id;
    type Mor = Path<Id, Id>;

    fn has_ob(&self, x: &Self::Ob) -> bool {
        self.category.has_ob(x)
    }
    fn has_mor(&self, m: &Self::Mor) -> bool {
        self.category.has_mor(m)
    }
    fn dom(&self, m: &Self::Mor) -> Self::Ob {
        self.category.dom(m)
    }
    fn cod(&self, m: &Self::Mor) -> Self::Ob {
        self.category.cod(m)
    }
    fn compose(&self, path: Path<Self::Ob, Self::Mor>) -> Self::Mor {
        self.category.compose(path)
    }
}

impl<Id, Cat> FgCategory for DiscreteDblModel<Id, Cat>
where
    Id: Eq + Clone + Hash,
    Cat: FgCategory,
    Cat::Ob: Hash,
    Cat::Mor: Hash,
{
    type ObGen = Id;
    type MorGen = Id;

    fn object_generators(&self) -> impl Iterator<Item = Self::ObGen> {
        self.category.object_generators()
    }

    fn morphism_generators(&self) -> impl Iterator<Item = Self::MorGen> {
        self.category.morphism_generators()
    }

    fn morphism_generator_dom(&self, f: &Self::MorGen) -> Self::Ob {
        self.category.morphism_generator_dom(f)
    }

    fn morphism_generator_cod(&self, f: &Self::MorGen) -> Self::Ob {
        self.category.morphism_generator_cod(f)
    }
}

impl<Id, Cat> DblModel for DiscreteDblModel<Id, Cat>
where
    Id: Eq + Clone + Hash,
    Cat: FgCategory,
    Cat::Ob: Hash,
    Cat::Mor: Hash,
{
    type ObType = Cat::Ob;
    type MorType = Cat::Mor;
    type ObOp = Cat::Ob;
    type MorOp = Cat::Mor;
    type Theory = DiscreteDblTheory<Cat>;

    fn theory(&self) -> &Self::Theory {
        &self.theory
    }

    fn ob_act(&self, x: Self::Ob, _: &Self::ObOp) -> Self::Ob {
        x
    }
    fn mor_act(&self, m: Self::Mor, _: &Self::MorOp) -> Self::Mor {
        m
    }

    fn ob_type(&self, ob: &Self::Ob) -> Self::ObType {
        self.ob_gen_type(ob)
    }
    fn mor_type(&self, mor: &Self::Mor) -> Self::MorType {
        let types = mor.clone().map(|x| self.ob_gen_type(&x), |m| self.mor_gen_type(&m));
        self.theory.compose_types(types)
    }
}

impl<Id, Cat> FgDblModel for DiscreteDblModel<Id, Cat>
where
    Id: Eq + Clone + Hash,
    Cat: FgCategory,
    Cat::Ob: Hash,
    Cat::Mor: Hash,
{
    fn ob_gen_type(&self, ob: &Self::ObGen) -> Self::ObType {
        self.ob_types.apply(ob).cloned().expect("Object should have type")
    }
    fn mor_gen_type(&self, mor: &Self::MorGen) -> Self::MorType {
        self.mor_types.apply(mor).cloned().expect("Morphism should have type")
    }

    fn object_generators_with_type(&self, typ: &Self::ObType) -> impl Iterator<Item = Self::ObGen> {
        self.ob_types.preimage(typ)
    }
    fn morphism_generators_with_type(
        &self,
        typ: &Self::MorType,
    ) -> impl Iterator<Item = Self::MorGen> {
        self.mor_types.preimage(typ)
    }
}

impl<Id, Cat> Validate for DiscreteDblModel<Id, Cat>
where
    Id: Eq + Clone + Hash,
    Cat: FgCategory,
    Cat::Ob: Hash,
    Cat::Mor: Hash,
{
    type ValidationError = InvalidDiscreteDblModel<Id>;

    fn validate(&self) -> Result<(), nonempty::NonEmpty<Self::ValidationError>> {
        validate::wrap_errors(self.iter_invalid())
    }
}

/** A failure of a model of a discrete double theory to be well defined.

TODO: Missing case that equation has different composite morphism types on left
and right hand sides.
*/
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "tag", content = "content"))]
#[cfg_attr(feature = "serde-wasm", derive(Tsify))]
#[cfg_attr(feature = "serde-wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum InvalidDiscreteDblModel<Id> {
    /// Domain of basic morphism is undefined or invalid.
    Dom(Id),

    /// Codomain of basic morphism is missing or invalid.
    Cod(Id),

    /// Basic object has invalid object type.
    ObType(Id),

    /// Basic morphism has invalid morphism type.
    MorType(Id),

    /// Domain of basic morphism has type incompatible with morphism type.
    DomType(Id),

    /// Codomain of basic morphism has type incompatible with morphism type.
    CodType(Id),

    /// Equation has left hand side that is not a well defined path.
    EqLhs(Id),

    /// Equation has right hand side that is not a well defined path.
    EqRhs(Id),

    /// Equation has different sources on left and right hand sides.
    EqSrc(Id),

    /// Equation has different sources on left and right hand sides.
    EqTgt(Id),
}

/// Object in a model of a discrete tabulator theory.
#[derive(Clone, PartialEq, Eq)]
pub enum TabOb<V, E> {
    /// Basic or generating object.
    Basic(V),

    /// A morphism viewed as an object of a tabulator.
    Tabulated(Box<TabMor<V, E>>),
}

impl<V, E> From<V> for TabOb<V, E> {
    fn from(value: V) -> Self {
        TabOb::Basic(value)
    }
}

/** "Edge" in a model of a discrete tabulator theory.

Morphisms of these two forms generate all the morphisms in the model.
 */
#[derive(Clone, PartialEq, Eq)]
pub enum TabEdge<V, E> {
    /// Basic morphism between any two objects.
    Basic(E),

    /// Generating morphism between tabulated morphisms, a commuting square.
    Square {
        /// The domain, a tabulated morphism.
        dom: Box<TabMor<V, E>>,

        /// The codomain, a tabulated morphism.
        cod: Box<TabMor<V, E>>,

        /// Edge that acts by pre-composition onto codomain.
        pre: Box<TabEdge<V, E>>,

        /// Edge that acts by post-composition onto domain.
        post: Box<TabEdge<V, E>>,
    },
}

impl<V, E> From<E> for TabEdge<V, E> {
    fn from(value: E) -> Self {
        TabEdge::Basic(value)
    }
}

/// Morphism in a model of a discrete tabulator theory.
pub type TabMor<V, E> = Path<TabOb<V, E>, TabEdge<V, E>>;

impl<V, E> From<E> for TabMor<V, E> {
    fn from(value: E) -> Self {
        Path::single(value.into())
    }
}

#[derive(Clone, Derivative)]
#[derivative(Default(bound = ""))]
#[derivative(PartialEq(bound = "V: Eq + Hash, E: Eq + Hash"))]
#[derivative(Eq(bound = "V: Eq + Hash, E: Eq + Hash"))]
struct DiscreteTabGenerators<V, E> {
    objects: HashFinSet<V>,
    morphisms: HashFinSet<E>,
    dom: HashColumn<E, TabOb<V, E>>,
    cod: HashColumn<E, TabOb<V, E>>,
}

impl<V, E> Graph for DiscreteTabGenerators<V, E>
where
    V: Eq + Clone + Hash,
    E: Eq + Clone + Hash,
{
    type V = TabOb<V, E>;
    type E = TabEdge<V, E>;

    fn has_vertex(&self, ob: &Self::V) -> bool {
        match ob {
            TabOb::Basic(v) => self.objects.contains(v),
            TabOb::Tabulated(p) => (*p).contained_in(self),
        }
    }

    fn has_edge(&self, edge: &Self::E) -> bool {
        match edge {
            TabEdge::Basic(e) => self.morphisms.contains(e),
            TabEdge::Square {
                dom,
                cod,
                pre,
                post,
            } => {
                if !(dom.contained_in(self) && cod.contained_in(self)) {
                    return false;
                }
                let path1 = dom.clone().concat_in(self, Path::single(*post.clone()));
                let path2 = Path::single(*pre.clone()).concat_in(self, *cod.clone());
                path1.is_some() && path2.is_some() && path1 == path2
            }
        }
    }

    fn src(&self, edge: &Self::E) -> Self::V {
        match edge {
            TabEdge::Basic(e) => {
                self.dom.apply(e).cloned().expect("Domain of morphism should be defined")
            }
            TabEdge::Square { dom, .. } => TabOb::Tabulated(dom.clone()),
        }
    }

    fn tgt(&self, edge: &Self::E) -> Self::V {
        match edge {
            TabEdge::Basic(e) => {
                self.cod.apply(e).cloned().expect("Codomain of morphism should be defined")
            }
            TabEdge::Square { cod, .. } => TabOb::Tabulated(cod.clone()),
        }
    }
}

/** A finitely presented model of a discrete tabulator theory.

A **model** of a [discrete tabulator theory](super::theory::DiscreteTabTheory)
is a normal lax functor from the theory into the double category of profunctors
that preserves tabulators. For the definition of "preserving tabulators," see
the dev docs.
 */
#[derive(Clone, Derivative)]
#[derivative(PartialEq(bound = "Id: Eq + Hash, ThId: Eq + Hash"))]
#[derivative(Eq(bound = "Id: Eq + Hash, ThId: Eq + Hash"))]
pub struct DiscreteTabModel<Id, ThId, S = RandomState> {
    #[derivative(PartialEq(compare_with = "Arc::ptr_eq"))]
    theory: Arc<DiscreteTabTheory<ThId, ThId, S>>,
    generators: DiscreteTabGenerators<Id, Id>,
    // TODO: Equations
    ob_types: IndexedHashColumn<Id, TabObType<ThId, ThId>>,
    mor_types: IndexedHashColumn<Id, TabMorType<ThId, ThId>>,
}

/// A model of a discrete tabulator theory where both theory and model have keys
/// of type `Ustr`.
pub type UstrDiscreteTabModel = DiscreteTabModel<Ustr, Ustr, BuildHasherDefault<IdentityHasher>>;

impl<Id, ThId, S> DiscreteTabModel<Id, ThId, S>
where
    Id: Eq + Clone + Hash,
    ThId: Eq + Clone + Hash,
    S: BuildHasher,
{
    /// Creates an empty model of the given theory.
    pub fn new(theory: Arc<DiscreteTabTheory<ThId, ThId, S>>) -> Self {
        Self {
            theory,
            generators: Default::default(),
            ob_types: Default::default(),
            mor_types: Default::default(),
        }
    }

    /// Convenience method to turn a morphism into an object.
    pub fn tabulated(&self, mor: TabMor<Id, Id>) -> TabOb<Id, Id> {
        TabOb::Tabulated(Box::new(mor))
    }

    /// Convenience method to turn a morphism generator into an object.
    pub fn tabulated_gen(&self, f: Id) -> TabOb<Id, Id> {
        self.tabulated(Path::single(TabEdge::Basic(f)))
    }

    /// Adds a basic object to the model.
    pub fn add_ob(&mut self, x: Id, typ: TabObType<ThId, ThId>) -> bool {
        self.ob_types.set(x.clone(), typ);
        self.generators.objects.insert(x)
    }

    /// Adds a basic morphism to the model.
    pub fn add_mor(
        &mut self,
        f: Id,
        dom: TabOb<Id, Id>,
        cod: TabOb<Id, Id>,
        typ: TabMorType<ThId, ThId>,
    ) -> bool {
        self.mor_types.set(f.clone(), typ);
        self.generators.dom.set(f.clone(), dom);
        self.generators.cod.set(f.clone(), cod);
        self.generators.morphisms.insert(f)
    }
}

impl<Id, ThId> Category for DiscreteTabModel<Id, ThId>
where
    Id: Eq + Clone + Hash,
{
    type Ob = TabOb<Id, Id>;
    type Mor = TabMor<Id, Id>;

    fn has_ob(&self, x: &Self::Ob) -> bool {
        self.generators.has_vertex(x)
    }
    fn has_mor(&self, path: &Self::Mor) -> bool {
        path.contained_in(&self.generators)
    }
    fn dom(&self, path: &Self::Mor) -> Self::Ob {
        path.src(&self.generators)
    }
    fn cod(&self, path: &Self::Mor) -> Self::Ob {
        path.tgt(&self.generators)
    }

    fn compose(&self, path: Path<Self::Ob, Self::Mor>) -> Self::Mor {
        path.flatten_in(&self.generators).expect("Paths should be composable")
    }
}

impl<Id, ThId> FgCategory for DiscreteTabModel<Id, ThId>
where
    Id: Eq + Clone + Hash,
{
    type ObGen = Id;
    type MorGen = Id;

    fn object_generators(&self) -> impl Iterator<Item = Self::ObGen> {
        self.generators.objects.iter()
    }
    fn morphism_generators(&self) -> impl Iterator<Item = Self::MorGen> {
        self.generators.morphisms.iter()
    }

    fn morphism_generator_dom(&self, f: &Self::MorGen) -> Self::Ob {
        self.generators.dom.apply(f).cloned().expect("Domain should be defined")
    }
    fn morphism_generator_cod(&self, f: &Self::MorGen) -> Self::Ob {
        self.generators.cod.apply(f).cloned().expect("Codomain should be defined")
    }
}

impl<Id, ThId> DblModel for DiscreteTabModel<Id, ThId>
where
    Id: Eq + Clone + Hash,
    ThId: Eq + Clone + Hash,
{
    type ObType = TabObType<ThId, ThId>;
    type MorType = TabMorType<ThId, ThId>;
    type ObOp = TabObOp<ThId, ThId>;
    type MorOp = TabMorOp<ThId, ThId>;
    type Theory = DiscreteTabTheory<ThId, ThId>;

    fn theory(&self) -> &Self::Theory {
        &self.theory
    }

    fn ob_type(&self, ob: &Self::Ob) -> Self::ObType {
        match ob {
            TabOb::Basic(x) => self.ob_gen_type(x),
            TabOb::Tabulated(m) => TabObType::Tabulator(Box::new(self.mor_type(m))),
        }
    }

    fn mor_type(&self, mor: &Self::Mor) -> Self::MorType {
        let types = mor.clone().map(
            |x| self.ob_type(&x),
            |edge| match edge {
                TabEdge::Basic(f) => self.mor_gen_type(&f),
                TabEdge::Square { dom, .. } => {
                    let typ = self.mor_type(&dom); // == self.mor_type(&cod)
                    TabMorType::Hom(Box::new(TabObType::Tabulator(Box::new(typ))))
                }
            },
        );
        self.theory.compose_types(types)
    }

    fn ob_act(&self, ob: Self::Ob, op: &Self::ObOp) -> Self::Ob {
        // Should we type check more rigorously here and in `mor_act`?
        match (ob, op) {
            (ob, TabObOp::Id(_)) => ob,
            (TabOb::Tabulated(m), TabObOp::ProjSrc(_)) => self.dom(&m),
            (TabOb::Tabulated(m), TabObOp::ProjTgt(_)) => self.cod(&m),
            _ => panic!("Ill-typed application of object operation"),
        }
    }

    fn mor_act(&self, mor: Self::Mor, op: &Self::MorOp) -> Self::Mor {
        match (mor, op) {
            (mor, TabMorOp::Id(_)) => mor,
            _ => panic!("Non-identity morphism operations not implemented"),
        }
    }
}

impl<Id, ThId> FgDblModel for DiscreteTabModel<Id, ThId>
where
    Id: Eq + Clone + Hash,
    ThId: Eq + Clone + Hash,
{
    fn ob_gen_type(&self, ob: &Self::ObGen) -> Self::ObType {
        self.ob_types.apply(ob).cloned().expect("Object should have type")
    }
    fn mor_gen_type(&self, mor: &Self::MorGen) -> Self::MorType {
        self.mor_types.apply(mor).cloned().expect("Morphism should have type")
    }
}

#[cfg(test)]
mod tests {
    use ustr::ustr;

    use super::*;
    use crate::one::fin_category::FinMor;
    use crate::stdlib::{models::*, theories::*};

    #[test]
    fn validate_discrete_dbl_model() {
        let th = Arc::new(th_schema());
        let mut model = DiscreteDblModel::new(th.clone());
        let entity = ustr("entity");
        model.add_ob(entity, ustr("NotObType"));
        assert_eq!(model.validate().unwrap_err().len(), 1);

        let mut model = DiscreteDblModel::new(th.clone());
        model.add_ob(entity, ustr("Entity"));
        model.add_mor(ustr("map"), entity, entity, FinMor::Generator(ustr("NotMorType")));
        assert_eq!(model.validate().unwrap_err().len(), 1);

        let mut model = DiscreteDblModel::new(th);
        model.add_ob(entity, ustr("Entity"));
        model.add_ob(ustr("type"), ustr("AttrType"));
        model.add_mor(ustr("a"), entity, ustr("type"), FinMor::Generator(ustr("Attr")));
        assert!(model.validate().is_ok());
        model.add_mor(ustr("b"), entity, ustr("type"), FinMor::Id(ustr("Entity")));
        assert_eq!(model.validate().unwrap_err().len(), 1);

        assert!(model.is_free());
        let peq = PathEq::new(Path::single(ustr("a")), Path::single(ustr("b")));
        model.add_equation(ustr("e"), peq);
        assert!(!model.is_free());
    }

    #[test]
    fn infer_discrete_dbl_model() {
        let th = Arc::new(th_schema());
        let mut model = DiscreteDblModel::new(th.clone());
        model.add_mor(ustr("attr"), ustr("entity"), ustr("type"), FinMor::Generator(ustr("Attr")));
        model.infer_missing();
        assert_eq!(model, walking_attr(th));
    }
}
