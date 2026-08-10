import { Model, Nb, type ModelDocument } from "catcolab-document-methods";
import type { MorType, ObType } from "catcolab-document-types";

const claim: ObType = { tag: "Basic", content: "Claim" };
const outcome: ObType = { tag: "Basic", content: "Outcome" };

/** A concrete PredictionMarket document for the current Haiku HDMI evidence. */
export function newHdmiBountyModel(): ModelDocument {
    const doc = Model.newModelDocument({ theory: "prediction-market" });
    doc.name = "Haiku HDMI evidence bounty";

    const objects = new Map<string, string>();
    const addObject = (name: string, obType: ObType) => {
        const decl = Model.newObjectDecl(obType);
        decl.name = name;
        objects.set(name, decl.id);
        Nb.appendCell(doc.notebook, Nb.newFormalCell(decl));
    };
    const addMorphism = (name: string, dom: string, cod: string, morType: MorType) => {
        const decl = Model.newMorphismDecl(morType);
        decl.name = name;
        decl.dom = { tag: "Basic", content: objects.get(dom)! };
        decl.cod = { tag: "Basic", content: objects.get(cod)! };
        Nb.appendCell(doc.notebook, Nb.newFormalCell(decl));
    };

    addObject("visible output", claim);
    addObject("mode list", claim);
    addObject("forced wrong mode", claim);
    addObject("live EDID", claim);
    addObject("sink identity", claim);
    addObject("completion", outcome);

    addMorphism("output observed", "visible output", "completion", {
        tag: "Basic",
        content: "Settles",
    });
    addMorphism("mode observed", "mode list", "completion", {
        tag: "Basic",
        content: "Settles",
    });
    addMorphism("wrong mode rejected", "forced wrong mode", "completion", {
        tag: "Basic",
        content: "SettlesAgainst",
    });
    addMorphism("EDID identifies sink", "live EDID", "sink identity", {
        tag: "Hom",
        content: claim,
    });
    addMorphism("mode enables output", "mode list", "visible output", {
        tag: "Hom",
        content: claim,
    });

    return doc;
}
