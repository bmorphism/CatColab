import { Repo } from "@automerge/automerge-repo";
import { afterAll, expect, test } from "vitest";

import { stdTheories } from "..";
import { ModelLibrary } from "../../model/model_library";
import { completionRows } from "../analyses/bounty_completion_logic";
import { newHdmiBountyModel } from "./hdmi_bounty";

const repo = new Repo();
const models = ModelLibrary.withRepo(repo, stdTheories);
afterAll(() => models.destroy());

test("HDMI bounty document elaborates into the completion analysis", async () => {
    const handle = repo.create(newHdmiBountyModel());
    const getEntry = await models.getElaboratedModel(handle.documentId);
    const entry = getEntry();
    expect(entry?.validatedModel.tag).toBe("Valid");
    if (entry?.validatedModel.tag !== "Valid") {
        return;
    }

    expect(
        completionRows(entry.validatedModel.model).map(({ label, state, related }) => ({
            label,
            state,
            related,
        })),
    ).toEqual([
        { label: "visible output", state: "confirmed", related: [] },
        {
            label: "mode list",
            state: "confirmed",
            related: [{ label: "visible output", polarity: "positive" }],
        },
        { label: "forced wrong mode", state: "rejected", related: [] },
        {
            label: "live EDID",
            state: "open",
            related: [{ label: "sink identity", polarity: "positive" }],
        },
        { label: "sink identity", state: "open", related: [] },
    ]);
});
