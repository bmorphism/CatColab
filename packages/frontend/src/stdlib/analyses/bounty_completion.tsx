import { createMemo, For, Show } from "solid-js";

import { BlockTitle } from "catcolab-ui-components";
import type { ModelAnalysisProps } from "../../analysis";
import { completionRows } from "./bounty_completion_logic";

/** Evidence-preserving bounty completion view for a PredictionMarket model. */
export default function BountyCompletion(props: ModelAnalysisProps<Record<string, never>>) {
    const rows = createMemo(() => {
        const model = props.liveModel.elaboratedModel();
        return model ? completionRows(model) : [];
    });

    return (
        <div>
            <BlockTitle title="Bounty completion" />
            <Show
                when={rows().length}
                fallback={<p>Add Claim objects to define acceptance criteria.</p>}
            >
                <table>
                    <thead>
                        <tr>
                            <th>Criterion</th>
                            <th>Evidence state</th>
                            <th>Related criteria</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For each={rows()}>
                            {(row) => (
                                <tr>
                                    <td>{row.label}</td>
                                    <td>{row.state}</td>
                                    <td>
                                        {row.related
                                            .map(({ label, polarity }) => `${polarity} → ${label}`)
                                            .join(", ") || "—"}
                                    </td>
                                </tr>
                            )}
                        </For>
                    </tbody>
                </table>
            </Show>
        </div>
    );
}
