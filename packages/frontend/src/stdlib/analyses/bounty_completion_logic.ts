import type { DblModel } from "catlog-wasm";

export type CompletionState = "confirmed" | "rejected" | "conflicted" | "open";

export type CompletionRow = {
    id: string;
    label: string;
    state: CompletionState;
    related: Array<{ label: string; polarity: "positive" | "negative" }>;
};

/** Classify prediction-market claims by their settlement at `completion`. */
export function completionRows(model: DblModel): CompletionRow[] {
    const completion = model
        .obGeneratorsWithType({ tag: "Basic", content: "Outcome" })
        .find((id) => model.obPresentation(id).label?.join(".") === "completion");
    const settlements = new Map<string, CompletionState>();
    const relations = new Map<
        string,
        Array<{ target: string; polarity: "positive" | "negative" }>
    >();
    for (const id of model.morGenerators()) {
        const mor = model.morPresentation(id);
        if (mor?.dom.tag === "Basic" && mor.cod.tag === "Basic") {
            const polarity =
                mor.morType.tag === "Hom" && mor.morType.content.tag === "Basic"
                    ? "positive"
                    : mor.morType.tag === "Basic" && mor.morType.content === "Negative"
                      ? "negative"
                      : undefined;
            if (polarity) {
                const outgoing = relations.get(mor.dom.content) ?? [];
                outgoing.push({ target: mor.cod.content, polarity });
                relations.set(mor.dom.content, outgoing);
            }
        }
        if (
            !(mor?.dom.tag === "Basic" && mor.cod.tag === "Basic" && mor.cod.content === completion)
        ) {
            continue;
        }
        let next: CompletionState | undefined;
        if (mor.morType.tag === "Basic" && mor.morType.content === "Settles") {
            next = "confirmed";
        } else if (mor.morType.tag === "Basic" && mor.morType.content === "SettlesAgainst") {
            next = "rejected";
        }
        if (next) {
            const previous = settlements.get(mor.dom.content);
            settlements.set(mor.dom.content, previous && previous !== next ? "conflicted" : next);
        }
    }

    return model.obGeneratorsWithType({ tag: "Basic", content: "Claim" }).map((id) => {
        const ob = model.obPresentation(id);
        return {
            id,
            label: ob.label?.join(".") || id,
            state: settlements.get(id) ?? "open",
            related: (relations.get(id) ?? []).map(({ target, polarity }) => ({
                label: model.obPresentation(target).label?.join(".") || target,
                polarity,
            })),
        };
    });
}
