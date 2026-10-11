use openmatrix9_rust::layers::om9_layer_effective_selectable;

#[test]
fn gumball_requires_visible_selectable_unhidden_unlocked_source() {
    for visible in [false, true] {
        for selectable in [false, true] {
            for hidden in [false, true] {
                for locked in [false, true] {
                    assert_eq!(
                        om9_layer_effective_selectable(visible, selectable, hidden, locked),
                        visible && selectable && !hidden && !locked,
                        "visible={visible} selectable={selectable} hidden={hidden} locked={locked}"
                    );
                }
            }
        }
    }
}
