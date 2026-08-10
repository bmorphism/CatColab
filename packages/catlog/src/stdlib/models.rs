//! Standard library of models of double theories.

use std::rc::Rc;

use crate::dbl::{model::*, theory::*};
use crate::one::{Path, QualifiedPath};
use crate::zero::{QualifiedName, name};

/// The positive self-loop.
///
/// A signed graph or free [signed category](super::theories::th_signed_category),
/// possibly with delays or indeterminates.
pub fn positive_loop(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    loop_of_type(th, name("Object"), Path::Id(name("Object")))
}

/// The negative self-loop.
///
/// A signed graph or free [signed category](super::theories::th_signed_category),
/// possibly with delays or indeterminates.
pub fn negative_loop(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    loop_of_type(th, name("Object"), name("Negative").into())
}

/// The delayed positive self-loop.
///
/// A free [delayable signed category](super::theories::th_delayable_signed_category).
pub fn delayed_positive_loop(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    loop_of_type(th, name("Object"), name("Slow").into())
}

/// The delayed negative self-loop.
///
/// A free [delayable signed category](super::theories::th_delayable_signed_category).
pub fn delayed_negative_loop(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    loop_of_type(th, name("Object"), Path::pair(name("Negative"), name("Slow")))
}

/// The Dutch-book loop: a sign-inconsistent cycle of claims.
///
/// A free model of the [prediction market
/// theory](super::theories::th_prediction_market). As a motif, occurrences of
/// this loop in a market model are cycles of conditional exposures with net
/// negative sign: exactly the cycles that admit a Dutch book, i.e. nontrivial
/// classes in the first sign cohomology of the market graph.
pub fn dutch_book_loop(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    loop_of_type(th, name("Claim"), name("Negative").into())
}

/// The coherent (positive) claim loop in a prediction market.
///
/// A free model of the [prediction market
/// theory](super::theories::th_prediction_market): a cycle of conditional
/// exposures with net positive sign, which constrains prices without
/// contradiction.
pub fn coherent_claim_loop(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    loop_of_type(th, name("Claim"), Path::Id(name("Claim")))
}

/// Verdict assigned to one criterion of a bounty completion object.
///
/// `Open` is deliberately represented by the absence of a settlement
/// morphism.  It must not be collapsed into either success or failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionVerdict {
    /// Evidence satisfies the criterion.
    Confirmed,
    /// Evidence disproves the criterion.
    Rejected,
    /// Available evidence does not settle the criterion.
    Open,
}

/// Construct the expected-completion object for an evidence-backed bounty.
///
/// Each named criterion is a `Claim`. Confirmed criteria settle for the
/// shared `completion` outcome, rejected criteria settle against it, and open
/// criteria have no settlement edge. Thus the returned market is a view of
/// the evidence currently available, not an oracle that manufactures a final
/// answer.
pub fn bounty_completion<I>(
    th: Rc<DiscreteDblTheory>,
    criteria: I,
) -> DiscreteDblModel
where
    I: IntoIterator<Item = (QualifiedName, CompletionVerdict)>,
{
    let mut market = DiscreteDblModel::new(th);
    let completion = name("completion");
    market.add_ob(completion.clone(), name("Outcome"));

    for (criterion, verdict) in criteria {
        market.add_ob(criterion.clone(), name("Claim"));
        let settlement = match verdict {
            CompletionVerdict::Confirmed => Some(("confirmed", "Settles")),
            CompletionVerdict::Rejected => Some(("rejected", "SettlesAgainst")),
            CompletionVerdict::Open => None,
        };
        if let Some((prefix, mor_type)) = settlement {
            let settlement_name = format!("{prefix}_{criterion}");
            market.add_mor(
                name(settlement_name.as_str()),
                criterion,
                completion.clone(),
                name(mor_type).into(),
            );
        }
    }
    market
}

/// Creates a self-loop with given object and morphism types.
fn loop_of_type(
    th: Rc<DiscreteDblTheory>,
    ob_type: QualifiedName,
    mor_type: QualifiedPath,
) -> DiscreteDblModel {
    let mut model = DiscreteDblModel::new(th);
    model.add_ob(name("x"), ob_type);
    model.add_mor(name("loop"), name("x"), name("x"), mor_type);
    model
}

/// The positive feedback loop between two objects.
///
/// A signed graph or free [signed category](super::theories::th_signed_category).
pub fn positive_feedback(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    let mut model = DiscreteDblModel::new(th);
    model.add_ob(name("x"), name("Object"));
    model.add_ob(name("y"), name("Object"));
    model.add_mor(name("positive1"), name("x"), name("y"), Path::Id(name("Object")));
    model.add_mor(name("positive2"), name("y"), name("x"), Path::Id(name("Object")));
    model
}

/// The negative feedback loop between two objects.
///
/// A signed graph or free [signed category](super::theories::th_signed_category).
pub fn negative_feedback(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    let mut model = DiscreteDblModel::new(th);
    model.add_ob(name("x"), name("Object"));
    model.add_ob(name("y"), name("Object"));
    model.add_mor(name("positive"), name("x"), name("y"), Path::Id(name("Object")));
    model.add_mor(name("negative"), name("y"), name("x"), name("Negative").into());
    model
}

/// The "walking attribute" schema.
///
/// A schema with one entity type, one attribute type, and one attribute.
pub fn walking_attr(th: Rc<DiscreteDblTheory>) -> DiscreteDblModel {
    let mut model = DiscreteDblModel::new(th);
    model.add_ob(name("entity"), name("Entity"));
    model.add_ob(name("type"), name("AttrType"));
    model.add_mor(name("attr"), name("entity"), name("type"), name("Attr").into());
    model
}

/// The "walking" backward link.
///
/// This is the free category with links that has a link from the codomain of a
/// morphism back to the morphism itself.
///
/// In system dynamics jargon, a backward link defines a "reinforcing loop,"
/// assuming the link has a positive effect on the flow. An example is an
/// infection flow an infectious disease model, where increasing the number of
/// infectives increases the rate of infection of the remaining susceptibles
/// (other things equal).
pub fn backward_link(th: Rc<DiscreteTabTheory>) -> DiscreteTabModel {
    backward_link_of_type(th, TabMorType::Basic(name("Link")))
}

/// The "walking" backward positive link.
///
/// This is the free category with signed links that has a positive link from
/// the codomain of a morphism back to the morphism itself.
pub fn positive_backward_link(th: Rc<DiscreteTabTheory>) -> DiscreteTabModel {
    // The type for positive links is just `Link`.
    backward_link_of_type(th, TabMorType::Basic(name("Link")))
}

/// The "walking" backward negative link.
///
/// This is the free category with signed links that has a negative link from
/// the codomain of a morphism back to the morphism itself.
pub fn negative_backward_link(th: Rc<DiscreteTabTheory>) -> DiscreteTabModel {
    backward_link_of_type(th, TabMorType::Basic(name("NegativeLink")))
}

fn backward_link_of_type(th: Rc<DiscreteTabTheory>, link_type: TabMorType) -> DiscreteTabModel {
    let ob_type = TabObType::Basic(name("Object"));
    let mut model = DiscreteTabModel::new(th.clone());
    model.add_ob(name("x"), ob_type.clone());
    model.add_ob(name("y"), ob_type.clone());
    model.add_mor(name("f"), name("x").into(), name("y").into(), th.hom_type(ob_type));
    model.add_mor(name("link"), name("y").into(), model.tabulated_gen(name("f")), link_type);
    model
}

/// A reaction involving three species, one playing the role of a catalyst.
///
/// A free symmetric monoidal category, viewed as a reaction network.
pub fn catalyzed_reaction(th: Rc<ModalDblTheory<Unital>>) -> ModalDblModel<Unital> {
    let (ob_type, op) = (ModalObType::new(name("Object")), name("tensor"));
    let mut model = ModalDblModel::new(th);
    model.add_ob(name("x"), ob_type.clone());
    model.add_ob(name("y"), ob_type.clone());
    model.add_ob(name("c"), ob_type.clone());
    let [x, y, c] = [name("x"), name("y"), name("c")].map(ModalOb::from);
    model.add_mor(
        name("f"),
        ModalOb::App(ModalOb::List(List::Symmetric, vec![x, c.clone()]).into(), op.clone()),
        ModalOb::App(ModalOb::List(List::Symmetric, vec![y, c]).into(), op),
        ModalMorType::Zero(ob_type),
    );
    model
}

/// The SIR model viewed as a reaction network.
pub fn sir_petri(th: Rc<ModalDblTheory<Unital>>) -> ModalDblModel<Unital> {
    let (ob_type, op) = (ModalObType::new(name("Object")), name("tensor"));
    let mut model = ModalDblModel::new(th);
    let (s, i, r) = (name("S"), name("I"), name("R"));
    model.add_ob(s.clone(), ob_type.clone());
    model.add_ob(i.clone(), ob_type.clone());
    model.add_ob(r.clone(), ob_type.clone());
    model.add_mor(
        name("infect"),
        ModalOb::App(
            ModalOb::List(List::Symmetric, vec![s.into(), i.clone().into()]).into(),
            op.clone(),
        ),
        ModalOb::App(
            ModalOb::List(List::Symmetric, vec![i.clone().into(), i.clone().into()]).into(),
            op.clone(),
        ),
        ModalMorType::Zero(ob_type.clone()),
    );
    model.add_mor(name("recover"), i.into(), r.into(), ModalMorType::Zero(ob_type));
    model
}

/// An example of Lotka–Volterra dynamics viewed as a non-unital theory for a symmetric multicategory.
pub fn lotka_volterra_dynamics(th: Rc<ModalDblTheory<NonUnital>>) -> ModalDblModel<NonUnital> {
    let ob_type = ModalObType::new(name("State"));
    let mor_type: ModalMorType = ModeApp::new(name("Contribution")).into();

    let mut model = ModalDblModel::new(th);
    // We're going to build a two-level predator-prey model, but where (in absence of signed
    // arrows) all interactions have positive coefficients.
    let (a, b, c) = (name("A"), name("B"), name("C"));

    model.add_ob(a.clone(), ob_type.clone());
    model.add_ob(b.clone(), ob_type.clone());
    model.add_ob(c.clone(), ob_type.clone());
    // The growth terms, corresponding to
    // dA/dt += g_A A
    // dB/dt += g_B B
    // dC/dt += g_C C
    model.add_mor(
        name("A_growth"),
        ModalOb::List(List::Symmetric, vec![a.clone().into()]),
        a.clone().into(),
        mor_type.clone(),
    );
    model.add_mor(
        name("B_growth"),
        ModalOb::List(List::Symmetric, vec![b.clone().into()]),
        b.clone().into(),
        mor_type.clone(),
    );
    model.add_mor(
        name("C_growth"),
        ModalOb::List(List::Symmetric, vec![c.clone().into()]),
        c.clone().into(),
        mor_type.clone(),
    );
    // The interaction terms, corresponding to
    // dB/dt += k_AB AB
    // dA/dt += k_BA AB
    // dC/dt += k_BC BC
    // dB/dt += k_CB BC
    model.add_mor(
        name("AB_interaction"),
        ModalOb::List(List::Symmetric, vec![a.clone().into(), b.clone().into()]),
        b.clone().into(),
        mor_type.clone(),
    );
    model.add_mor(
        name("BA_interaction"),
        ModalOb::List(List::Symmetric, vec![a.clone().into(), b.clone().into()]),
        a.clone().into(),
        mor_type.clone(),
    );
    model.add_mor(
        name("BC_interaction"),
        ModalOb::List(List::Symmetric, vec![b.clone().into(), c.clone().into()]),
        c.clone().into(),
        mor_type.clone(),
    );
    model.add_mor(
        name("CB_interaction"),
        ModalOb::List(List::Symmetric, vec![b.clone().into(), c.clone().into()]),
        b.clone().into(),
        mor_type,
    );

    model
}

#[cfg(test)]
mod tests {
    use super::super::theories::*;
    use super::*;
    use crate::one::category::FgCategory;
    use crate::validate::Validate;

    #[test]
    fn prediction_market_complex() {
        // A market complex observed live on Manifold (July 2026): the
        // "AutopoieticErgodicity" family of markets about one person. Manifold
        // stores these as *isolated* CPMM pools; every edge below is a modeling
        // commitment imposed on top, and the point of the test is that the
        // motif machinery audits those commitments.
        //
        // Claims (Manifold market ids):
        //   mating_attempt      i0k3il02tf  p=0.248
        //   duck_relationship   y64l9f6vmo  p=0.576
        //   roasted_duck        xmcklqvl8z  p=0.532
        //   penguin_relationship 3h8hsw6g93 p=0.250
        //   duck_tales          s5sl5n69gO  p=0.500
        //   obese_2030          lupOPAS02d  p=0.505 (deliberately isolated:
        //     no defensible coupling — sparsity is data, not failure)
        use crate::dbl::model_morphism::DiscreteDblModelMapping;

        let th = Rc::new(th_prediction_market());
        let mut market = DiscreteDblModel::new(th.clone());
        for claim in ["mating_attempt", "duck_relationship", "roasted_duck",
                      "penguin_relationship", "duck_tales", "obese_2030"] {
            market.add_ob(name(claim), name("Claim"));
        }
        market.add_ob(name("duck_tales_airs"), name("Outcome"));

        let pos = Path::Id(name("Claim"));
        let neg: QualifiedPath = name("Negative").into();
        // Commitments:
        market.add_mor(name("courtship"), name("mating_attempt"),
            name("duck_relationship"), pos.clone()); // courtship raises relationship
        market.add_mor(name("devotion"), name("duck_relationship"),
            name("roasted_duck"), neg.clone()); // partner-species taboo
        market.add_mor(name("taboo"), name("roasted_duck"),
            name("duck_relationship"), neg.clone()); // reciprocal foreclosure
        market.add_mor(name("affinity"), name("duck_relationship"),
            name("penguin_relationship"), pos.clone()); // bird-affinity generalizes
        market.add_mor(name("rivalry"), name("penguin_relationship"),
            name("duck_relationship"), neg.clone()); // exclusive affection
        market.add_mor(name("fodder"), name("duck_relationship"),
            name("duck_tales"), pos.clone()); // narrative fodder
        market.add_mor(name("airs"), name("duck_tales"),
            name("duck_tales_airs"), name("Settles").into());
        assert!(market.validate().is_ok());

        // Dutch-book audit: `affinity` and `rivalry` are *jointly* incoherent
        // (net negative cycle), even though each is individually defensible.
        // The motif finder must catch our own contradictory commitments.
        let dutch = dutch_book_loop(th.clone());
        let found = DiscreteDblModelMapping::morphisms(&dutch, &market).monic().find_all();
        assert!(!found.is_empty(), "affinity+rivalry should form a Dutch book");

        // The devotion/taboo 2-cycle is net positive: coherent, not a book.
        let coherent = coherent_claim_loop(th.clone());
        let found = DiscreteDblModelMapping::morphisms(&coherent, &market).monic().find_all();
        assert!(!found.is_empty(), "devotion+taboo should form a coherent loop");

        // Falsifiability: the counterfactual world without `affinity` must
        // have no Dutch book. (No retraction API: counterfactual worlds are
        // constructed, not mutated.)
        let mut repaired = DiscreteDblModel::new(th.clone());
        repaired.add_ob(name("duck_relationship"), name("Claim"));
        repaired.add_ob(name("penguin_relationship"), name("Claim"));
        repaired.add_mor(name("rivalry"), name("penguin_relationship"),
            name("duck_relationship"), neg.clone());
        assert!(repaired.validate().is_ok());
        let found = DiscreteDblModelMapping::morphisms(&dutch, &repaired).monic().find_all();
        assert!(found.is_empty(), "without affinity there is no book");
    }

    #[test]
    fn resolved_market_poles() {
        // Real settlements observed on Manifold (July 2026), exercising both
        // resolution morphisms with live data:
        //   u9n9LPUcqU "Will the sun rise tomorrow?" resolved YES at p=0.99
        //     (creator @bmorphism; Laplace's example, played straight)
        //   gqeymkhcmj "Will @bmorphism lose The Pentagon by end of 2024"
        //     resolved YES with ZERO bets ever placed: settlement without
        //     trading. The pole fires by creator fiat, independent of any
        //     market dynamics — which is why `Settles` is structure, not
        //     valuation.
        //   gysm9cbgov "...escape a manifold market addiction?" resolved NO
        //     at p=0.18: a reflexive claim (about the subject's relation to
        //     the market system containing it), settled against.
        let th = Rc::new(th_prediction_market());
        let mut market = DiscreteDblModel::new(th);
        market.add_ob(name("sun_rise"), name("Claim"));
        market.add_ob(name("pentagon_lost"), name("Claim"));
        market.add_ob(name("addiction_escape"), name("Claim"));
        market.add_ob(name("audit_2024"), name("Outcome"));
        market.add_mor(name("sun_yes"), name("sun_rise"),
            name("audit_2024"), name("Settles").into());
        market.add_mor(name("pentagon_yes"), name("pentagon_lost"),
            name("audit_2024"), name("Settles").into());
        market.add_mor(name("escape_no"), name("addiction_escape"),
            name("audit_2024"), name("SettlesAgainst").into());
        assert!(market.validate().is_ok());
    }

    #[test]
    fn hdmi_bounty_completion_preserves_unknowns() {
        let th = Rc::new(th_prediction_market());
        let mut market = bounty_completion(
            th,
            [
                (name("visible_output"), CompletionVerdict::Confirmed),
                (name("mode_list"), CompletionVerdict::Confirmed),
                (name("forced_wrong_mode"), CompletionVerdict::Rejected),
                (name("live_edid"), CompletionVerdict::Open),
                (name("sink_identity"), CompletionVerdict::Open),
            ],
        );
        market.add_mor(
            name("edid_identifies_sink"),
            name("live_edid"),
            name("sink_identity"),
            Path::Id(name("Claim")),
        );
        market.add_mor(
            name("mode_enables_output"),
            name("mode_list"),
            name("visible_output"),
            Path::Id(name("Claim")),
        );

        assert!(market.validate().is_ok());
        assert_eq!(market.ob_generators().count(), 6);
        assert_eq!(market.mor_generators().count(), 5);
    }

    #[test]
    fn signed_categories() {
        let th = Rc::new(th_signed_category());
        assert!(positive_loop(th.clone()).validate().is_ok());
        assert!(negative_loop(th.clone()).validate().is_ok());
        assert!(positive_feedback(th.clone()).validate().is_ok());
        assert!(negative_feedback(th.clone()).validate().is_ok());
    }

    #[test]
    fn delayable_signed_categories() {
        let th = Rc::new(th_delayable_signed_category());
        assert!(positive_loop(th.clone()).validate().is_ok());
        assert!(negative_loop(th.clone()).validate().is_ok());
        assert!(delayed_positive_loop(th.clone()).validate().is_ok());
        assert!(delayed_negative_loop(th.clone()).validate().is_ok());
    }

    #[test]
    fn schemas() {
        let th = Rc::new(th_schema());
        assert!(walking_attr(th).validate().is_ok());
    }

    #[test]
    fn categories_with_links() {
        let th = Rc::new(th_category_links());
        assert!(backward_link(th).validate().is_ok());
    }

    #[test]
    fn categories_with_signed_links() {
        let th = Rc::new(th_category_signed_links());
        assert!(positive_backward_link(th.clone()).validate().is_ok());
        assert!(negative_backward_link(th.clone()).validate().is_ok());
    }

    #[test]
    fn sym_monoidal_categories() {
        let th = Rc::new(th_sym_monoidal_category());
        assert!(catalyzed_reaction(th.clone()).validate().is_ok());
        assert!(sir_petri(th).validate().is_ok());
    }

    #[test]
    fn polynomial_ode_systems() {
        let th = Rc::new(th_polynomial_ode_system());
        assert!(lotka_volterra_dynamics(th.clone()).validate().is_ok());
    }
}
