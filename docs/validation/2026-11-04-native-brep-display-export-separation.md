# Retained B-rep display and STL source

An integration fixture creates one OCCT sphere and converts it to a retained native 3DM B-rep payload. It exports that same payload twice with the same linear/angular profile: once with its normal display mesh and once with a deliberately incomplete but structurally valid one-face display cache. Both STL files are byte-identical, contain more than one triangle and pass the receipt's indexed closedness checks. The focused test passed on E: with `-j 2`.

This demonstrates that this retained B-rep STL route re-tessellates the payload rather than consuming the display mesh for this fixture. The deliberately incomplete display is a separation probe, not an approved display-quality mode. It does not prove global surface deviation, topology identity, 3DM roundtrip equivalence or P1's general source/display ownership and file migration.
