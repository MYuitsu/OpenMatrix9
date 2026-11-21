# Top11 Ring Rail region picker

The Top11 button opens a compact Ring Rail bar below Projects, initially showing USA size 7. Clicking the region name opens the size systems. Set Region confirms the selection. The green arrow on the main bar creates the rail and leaves the controls available. Closing the bar before placement discards the session. The Tools builder and typed command retain their existing defaults and shape library.

The seven reference regions are USA, Britain, Japan, Swiss, European, France and German. VNM accepts an inside diameter in mm or an inside circumference in mm. Custom Ring Size supports both measures in mm and inches. Named sizes use the options in the region's size list. The US list keeps OM9's existing Stuller quarter-size range. The remaining ranges and diameter relationships were checked against the private MatrixCommon JewelryHelper size enumeration and conversion. The interaction follows the four user screenshots and the normalized Book 1 Ring Rail reference, pages 87-88.

## Windows verification

In a clean profile with the newly built Matrix native add-on enabled, the actual Top11 icon opened USA size 7 without adding a curve. Selecting VNM and confirming Set Region retained the physical diameter of 17.32 mm. Clicking the main green arrow created one circle with radius 8.66 mm at the world origin in the Front plane. The desktop controller reported one Undo entry. Undo returned the document to zero objects. The curve takes the document layer color.

The integration test drives pointer events against the application's rendered controls. It opens USA, selects VNM, checks that the draft has not changed the command, confirms the region, then changes the size to 20 mm through CMD. It verifies the view controls remain after placement, the exact 10 mm radius and Front axes, and document equality through Undo and Redo. The regional size test checks every listed increment against its diameter and a full geometry/history round trip for one size per named region.

The four Windows images are in [the capture directory](../../build/standalone-validation/ui-artifacts/2026-11-22-ring-rail). [Before opening](../../build/standalone-validation/ui-artifacts/2026-11-22-ring-rail/01-projects.png) · [USA size 7](../../build/standalone-validation/ui-artifacts/2026-11-22-ring-rail/02-rail-bar.png) · [Regional picker](../../build/standalone-validation/ui-artifacts/2026-11-22-ring-rail/03-regions-vnm.png) · [Placed circle](../../build/standalone-validation/ui-artifacts/2026-11-22-ring-rail/04-placed-vnm.png).

The focused Windows run passed eight `ring_tools` cases, eleven `shell_docking` cases and all five Matrix package checks. Cargo used the repository manifests, the existing external target and two build jobs. Each package ran separately to keep native export features independent.

The broad application and core library test builds still encounter older BRep fixture type errors. An independent shell test for unknown-command status returned Ready when it expected Error. It is also listed in the [earlier workspace regression report](2026-11-05-workspace-build-test-commit.md).
