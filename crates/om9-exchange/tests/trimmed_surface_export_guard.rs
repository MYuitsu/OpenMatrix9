// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits,geometry::Geometry,model::{Definition,Document,Entity},
    rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::{TransactionCoordinator,World}};
use om9_exchange::ExportCapture;

#[test]
fn three_dm_packet_refuses_trimmed_surface_in_root_or_definition() {
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    for nested in [false,true] {
        let mut doc=Document::with_template_tolerance("3dm trim");
        let root=if nested {let definition=Definition::new("trim definition",vec![entity.clone()]);let root=Entity::new("instance",Geometry::Instance(definition.id),None);
            doc.definitions.insert(definition.id,definition);root} else {entity.clone()};
        let root_id=root.id;doc.entities.insert(root_id,root);
        let plain=Entity::new("plain",Geometry::Point([0.,0.,0.]),None);let plain_id=plain.id;doc.entities.insert(plain_id,plain);
        let id=doc.id;let coordinator=TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
        let all=ExportCapture::all(&coordinator,id).unwrap();
        assert!(all.encode(&limits).is_err(),"trimmed source must not become its full surface");
        let selected=ExportCapture::selected(&coordinator,id,&[root_id]).unwrap();assert!(selected.encode(&limits).is_err());
        let unrelated=ExportCapture::selected(&coordinator,id,&[plain_id]).unwrap();assert!(unrelated.encode(&limits).is_ok());
    }
}
