// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    geometry::Unit,
    metal_weight::{
        DensityUnit, MM3_PER_CM3, MassBreakdown, MassPart, MetalEstimate, MetalMaterial,
        MetalMaterialRegistry, SolidMeasure, closed_solid_volume_mm3, grams_to_dwt,
        volume_mm3_from_project,
    },
};

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0),
        "expected {expected}, got {actual}"
    );
}

fn synthetic_registry() -> MetalMaterialRegistry {
    let mut registry = MetalMaterialRegistry::empty();
    registry
        .insert(MetalMaterial::test_only("synthetic_material", 10.0, "test fixture").unwrap())
        .unwrap();
    registry
        .insert(MetalMaterial::test_only("light_material", 8.0, "test fixture").unwrap())
        .unwrap();
    registry
}

#[test]
fn a_closed_box_volume_is_admitted_and_project_units_convert_to_mm() {
    close(closed_solid_volume_mm3(SolidMeasure::closed_mm3(100.0, 1)).unwrap(), 100.0);
    // 10 cm x 5 cm x 2 cm = 100 cm^3 = 100_000 mm^3.
    close(volume_mm3_from_project(100.0, Unit::Centimeter).unwrap(), 100_000.0);
    close(volume_mm3_from_project(100.0, Unit::Millimeter).unwrap(), 100.0);
}

#[test]
fn an_open_shell_or_an_empty_solid_never_gets_a_volume() {
    let open = SolidMeasure {
        volume: 100.0,
        unit: Unit::Millimeter,
        closed: false,
        solids: 1,
    };
    assert_eq!(
        closed_solid_volume_mm3(open),
        Err(Error::Invalid("metal weight requires a closed solid"))
    );
    let no_solid = SolidMeasure {
        closed: true,
        solids: 0,
        ..open
    };
    assert_eq!(
        closed_solid_volume_mm3(no_solid),
        Err(Error::Invalid("metal weight requires a closed solid"))
    );
}

#[test]
fn non_finite_and_zero_volumes_are_refused() {
    for volume in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            closed_solid_volume_mm3(SolidMeasure::closed_mm3(volume, 1)),
            Err(Error::Invalid("metal volume"))
        );
    }
    assert_eq!(
        volume_mm3_from_project(f64::NAN, Unit::Millimeter),
        Err(Error::Invalid("metal volume"))
    );
}

#[test]
fn a_synthetic_density_registers_with_provenance_and_the_cm3_factor() {
    let registry = synthetic_registry();
    let material = registry.get("synthetic_material").unwrap();
    assert_eq!(material.density_g_cm3, 10.0);
    assert_eq!(material.provenance, "test fixture");
    assert!(material.test_only);
    assert_eq!(material.density_kg_m3(), 10_000.0);
    assert_eq!(MM3_PER_CM3, 1000.0);
    assert_eq!(registry.get("SYNTHETIC_MATERIAL").unwrap().id, "synthetic_material");
}

#[test]
fn a_missing_material_and_an_impossible_density_are_refused() {
    let registry = synthetic_registry();
    assert_eq!(
        registry.get("missing").unwrap_err(),
        Error::Missing("metal material density")
    );
    assert_eq!(
        registry.get("  ").unwrap_err(),
        Error::Missing("metal material id")
    );
    assert_eq!(
        MetalMaterial::new(
            "bad",
            "Bad density",
            -10.0,
            DensityUnit::GramsPerCubicCentimeter,
            "fixture"
        )
        .unwrap_err(),
        Error::Invalid("metal material density")
    );
    assert_eq!(
        MetalMaterial::new(
            "misread",
            "kg/m3 read as g/cm3",
            15_500.0,
            DensityUnit::GramsPerCubicCentimeter,
            "fixture"
        )
        .unwrap_err(),
        Error::Invalid("metal material density")
    );
}

#[test]
fn a_kg_m3_density_is_converted_before_it_is_used() {
    let material = MetalMaterial::new(
        "kg_material",
        "10 g/cm^3 stated in kg/m^3",
        10_000.0,
        DensityUnit::KilogramsPerCubicMeter,
        "fixture",
    )
    .unwrap();
    close(material.density_g_cm3, 10.0);
    let estimate = MetalEstimate::new(100.0, &material).unwrap();
    close(estimate.grams, 1.0);
}

#[test]
fn mass_is_volume_over_one_thousand_times_density_in_grams_and_dwt() {
    let registry = synthetic_registry();
    let material = registry.get("synthetic_material").unwrap();
    let estimate = MetalEstimate::new(100.0, material).unwrap();
    close(estimate.grams, 1.0);
    close(estimate.dwt, 1.0 / 1.555_173_84);
    close(grams_to_dwt(1.0).unwrap(), 0.643_014_931_5);
    assert_eq!(estimate.uncertainty_grams, 0.0);
}

#[test]
fn a_builtin_material_carries_provenance_and_an_uncertainty_band() {
    let registry = MetalMaterialRegistry::builtin();
    let gold = registry.get("gold_18k_yellow").unwrap();
    assert_eq!(gold.name, "18k yellow gold");
    assert!(!gold.provenance.is_empty());
    assert_eq!(gold.uncertainty_percent, 1.0);
    let estimate = MetalEstimate::new(100.0, gold).unwrap();
    close(estimate.grams, 1.55);
    close(estimate.uncertainty_grams, 0.0155);
}

#[test]
fn multi_material_breakdown_sums_per_body_and_avoids_double_counting() {
    let registry = synthetic_registry();
    let parts = [
        MassPart::new("body-a", 100.0, "synthetic_material"),
        MassPart::new("body-b", 200.0, "light_material"),
    ];
    let breakdown = MassBreakdown::compute(&parts, &registry).unwrap();
    assert_eq!(breakdown.entries.len(), 2);
    close(breakdown.entries[0].grams, 1.0);
    close(breakdown.entries[1].grams, 1.6);
    close(breakdown.total_grams, 2.6);
    close(breakdown.total_dwt, 2.6 / 1.555_173_84);
    close(breakdown.total_volume_mm3, 300.0);

    let repeated = [
        MassPart::new("body-a", 100.0, "synthetic_material").with_instances(2),
        MassPart::new("body-b", 200.0, "light_material"),
    ];
    let counted = MassBreakdown::compute(&repeated, &registry).unwrap();
    close(counted.entries[0].grams, 2.0);
    close(counted.total_grams, 3.6);
}

#[test]
fn a_duplicate_body_id_without_an_explicit_count_is_refused() {
    let registry = synthetic_registry();
    let parts = [
        MassPart::new("body-a", 100.0, "synthetic_material"),
        MassPart::new("BODY-A", 100.0, "synthetic_material"),
    ];
    assert_eq!(
        MassBreakdown::compute(&parts, &registry),
        Err(Error::Invalid("duplicate metal body"))
    );
}

#[test]
fn a_breakdown_refuses_unknown_materials_and_bad_counts() {
    let registry = synthetic_registry();
    assert_eq!(
        MassBreakdown::compute(
            &[MassPart::new("body", 100.0, "missing")],
            &registry
        ),
        Err(Error::Missing("metal material density"))
    );
    assert_eq!(
        MassBreakdown::compute(
            &[MassPart::new("body", 100.0, "synthetic_material").with_instances(0)],
            &registry
        ),
        Err(Error::Invalid("metal body instance count"))
    );
    assert_eq!(
        MassBreakdown::compute(&[MassPart::new(" ", 100.0, "synthetic_material")], &registry),
        Err(Error::Missing("metal body id"))
    );
    assert!(MassBreakdown::compute(&[], &registry).is_err());
}
