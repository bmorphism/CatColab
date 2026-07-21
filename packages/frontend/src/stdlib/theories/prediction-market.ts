import { lazy } from "solid-js";

import { ThPredictionMarket } from "catlog-wasm";
import { Theory, type TheoryMeta } from "../../theory";
import * as analyses from "../analyses";

const ObjectCellEditor = lazy(() => import("../../model/object_cell_editor"));
const MorphismCellEditor = lazy(() => import("../../model/morphism_cell_editor"));

/** The theory of prediction markets.

A market graph is a signed category of open claims together with a settlement
pole: resolution edges send claims to settled outcomes. Coherent prices are the
harmonic (martingale) labelings extending the settled boundary; Dutch books are
sign-inconsistent claim cycles. Valuation is an analysis, never model data.
 */
export default function createPredictionMarketTheory(theoryMeta: TheoryMeta): Theory {
    const thPredictionMarket = new ThPredictionMarket();

    return new Theory({
        ...theoryMeta,
        theory: thPredictionMarket.theory(),
        onlyFreeModels: true,
        modelTypes: [
            {
                tag: "ObType",
                obType: { tag: "Basic", content: "Claim" },
                editor: ObjectCellEditor,
                name: "Claim",
                shortcut: ["C"],
                description: "Open claim: an unresolved question with a price",
            },
            {
                tag: "ObType",
                obType: { tag: "Basic", content: "Outcome" },
                editor: ObjectCellEditor,
                name: "Outcome",
                shortcut: ["O"],
                description: "Settled outcome: a resolved boundary event",
            },
            {
                tag: "MorType",
                morType: {
                    tag: "Hom",
                    content: { tag: "Basic", content: "Claim" },
                },
                editor: MorphismCellEditor,
                name: "Positive exposure",
                shortcut: ["P"],
                description: "Claims move in the same direction",
                arrowStyle: "plus",
                preferUnnamed: true,
            },
            {
                tag: "MorType",
                morType: { tag: "Basic", content: "Negative" },
                editor: MorphismCellEditor,
                name: "Negative exposure",
                shortcut: ["N"],
                description: "Claims move in opposite directions",
                arrowStyle: "minus",
                preferUnnamed: true,
            },
            {
                tag: "MorType",
                morType: { tag: "Basic", content: "Settles" },
                editor: MorphismCellEditor,
                name: "Settles for",
                shortcut: ["S"],
                description: "Claim resolves in favor of the outcome",
                preferUnnamed: true,
            },
            {
                tag: "MorType",
                morType: { tag: "Basic", content: "SettlesAgainst" },
                editor: MorphismCellEditor,
                name: "Settles against",
                shortcut: ["A"],
                description: "Claim resolves against the outcome",
                arrowStyle: "minus",
                preferUnnamed: true,
            },
        ],
        modelAnalyses: [
            analyses.modelGraph({
                id: "diagram",
                name: "Visualization",
                description: "Visualize the market graph",
                help: "visualization",
            }),
            analyses.motifFinding({
                id: "dutch-books",
                name: "Dutch books",
                description: "Find sign-inconsistent claim cycles (arbitrage)",
                help: "loops",
                findMotifs(model, options) {
                    return thPredictionMarket.dutchBooks(model, options);
                },
            }),
            analyses.motifFinding({
                id: "coherent-loops",
                name: "Coherent loops",
                description: "Find sign-consistent claim cycles",
                help: "loops",
                findMotifs(model, options) {
                    return thPredictionMarket.coherentLoops(model, options);
                },
            }),
        ],
    });
}
