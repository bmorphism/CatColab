import { describe, expect, test } from "vitest";

import type { DblModel } from "catlog-wasm";
import { completionRows } from "./bounty_completion_logic";

describe("bounty completion", () => {
    test("keeps confirmed, rejected, open, and conflicted criteria distinct", () => {
        const labels: Record<string, string> = {
            pole: "completion",
            visible: "visible output",
            wrong: "forced wrong mode",
            edid: "live EDID",
            identity: "sink identity",
        };
        const morphisms = {
            visibleYes: ["visible", "Settles"],
            wrongNo: ["wrong", "SettlesAgainst"],
            identityYes: ["identity", "Settles"],
            identityNo: ["identity", "SettlesAgainst"],
            edidSupportsIdentity: ["edid", "Positive", "identity"],
        } as const;
        const model = {
            obGeneratorsWithType: (type: { content: string }) =>
                type.content === "Outcome" ? ["pole"] : ["visible", "wrong", "edid", "identity"],
            obPresentation: (id: string) => ({ id, label: [labels[id]] }),
            morGenerators: () => Object.keys(morphisms),
            morPresentation: (id: keyof typeof morphisms) => ({
                dom: { tag: "Basic", content: morphisms[id][0] },
                cod: { tag: "Basic", content: morphisms[id][2] ?? "pole" },
                morType:
                    morphisms[id][1] === "Positive"
                        ? { tag: "Hom", content: { tag: "Basic", content: "Claim" } }
                        : { tag: "Basic", content: morphisms[id][1] },
            }),
        } as unknown as DblModel;

        expect(completionRows(model).map(({ label, state }) => [label, state])).toEqual([
            ["visible output", "confirmed"],
            ["forced wrong mode", "rejected"],
            ["live EDID", "open"],
            ["sink identity", "conflicted"],
        ]);
        expect(completionRows(model).find(({ label }) => label === "live EDID")?.related).toEqual([
            { label: "sink identity", polarity: "positive" },
        ]);
    });
});
