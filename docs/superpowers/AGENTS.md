# Scope of existing plans and designs

Existing dated FreeCAD/Qt-Coin/plugin plans and designs in this directory describe the historical host implementation. Their approval language, build steps and passes do not define the standalone target.

For new plans, use `../../OpenMatrix9_Codex_Spec_v2`, RCORE-00, CODE_GUIDE, BACKEND_POLICY and MATRIX9_UI_CONTRACT. Plan own Rust OM9 document/history/UI/scene, selected routine/native adapters and Matrix9 UI as currently described. QymCAD supplies code samples only; do not inherit its UI/workflow. Never fallback to v1.

Run legacy host steps only for an explicitly scoped maintenance/comparison task, using the parent shared SDK policy. Preserve old plan bodies and evidence; do not relabel historical completion as standalone acceptance.
