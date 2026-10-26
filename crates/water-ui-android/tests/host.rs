//! Drives the adapter's host half on whatever GPU the machine has, the path
//! the JNI surface takes on a phone. Each test returns early with a note when
//! no hardware device opens, as the infra GPU tests do.

use std::sync::OnceLock;

use water_ui_android::host::{CatalogueRequest, Engine, Timing};
use water_ui_infra::gpu::GpuContext;

/// One device for the binary (tests run on parallel threads); an engine per
/// test, since an engine is confined to the thread that drives it.
static CONTEXT: OnceLock<Option<GpuContext>> = OnceLock::new();

fn engine() -> Option<Engine> {
    let ctx = CONTEXT.get_or_init(|| match GpuContext::try_new() {
        Ok(Some(ctx)) if !ctx.is_software() => Some(ctx),
        _ => None,
    });
    ctx.clone()
        .map(|ctx| Engine::with_context(ctx).expect("engine"))
}

fn edge() -> CatalogueRequest<'static> {
    CatalogueRequest {
        artwork_id: "selection-edge",
        seed: 7,
        palette: "water",
        intensity: 0.7,
        detail: "medium",
        surface: "light",
        sim_resolution: 0,
    }
}

fn timing() -> Timing {
    Timing {
        duration_ms: 600.0,
        ..Timing::default()
    }
}

#[test]
fn a_reveal_paints_into_a_bitmap_buffer() {
    let Some(engine) = engine() else {
        eprintln!("skipped: no hardware GPU");
        return;
    };
    let mut instance = engine.catalogue_instance(&edge(), timing()).unwrap();
    let (w, h) = (32, 192);
    let mut blank = vec![0u8; w * h * 4];
    instance
        .render_into(w as u32, h as u32, &mut blank)
        .unwrap();

    instance.play();
    let mut moved = false;
    for _ in 0..120 {
        moved |= instance.advance_by_elapsed(1.0 / 60.0).unwrap();
        if instance.is_finished() {
            break;
        }
    }
    assert!(moved, "advancing never ran a tick");
    instance.finish_immediately().unwrap();
    assert!(instance.is_finished());

    let mut painted = vec![0u8; w * h * 4];
    instance
        .render_into(w as u32, h as u32, &mut painted)
        .unwrap();
    let covered = painted
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[3] > 32)
        .count();
    assert!(covered > w * h / 10, "only {covered} painted pixels");
    // Premultiplied: no channel may exceed its pixel's alpha.
    assert!(
        painted
            .as_chunks::<4>()
            .0
            .iter()
            .all(|px| px[0] <= px[3] && px[1] <= px[3] && px[2] <= px[3])
    );
    assert_ne!(blank, painted);
}

#[test]
fn the_live_cap_refuses_and_frees_slots() {
    let Some(engine) = engine() else {
        eprintln!("skipped: no hardware GPU");
        return;
    };
    engine.set_max_live_instances(1);
    let first = engine.catalogue_instance(&edge(), timing()).unwrap();
    let refused = engine.catalogue_instance(&edge(), timing()).err().unwrap();
    assert!(refused.message().starts_with("InstanceLimit:"), "{refused}");
    drop(first);
    assert_eq!(engine.live_instances(), 0);
    assert!(engine.catalogue_instance(&edge(), timing()).is_ok());
}

#[test]
fn bad_arguments_cross_as_coded_errors() {
    let Some(engine) = engine() else {
        eprintln!("skipped: no hardware GPU");
        return;
    };
    let unknown = CatalogueRequest {
        artwork_id: "no-such-art",
        ..edge()
    };
    let err = engine.catalogue_instance(&unknown, timing()).err().unwrap();
    assert!(err.message().starts_with("UnknownArtwork:"), "{err}");

    let bad_surface = CatalogueRequest {
        surface: "sepia",
        ..edge()
    };
    let err = engine
        .catalogue_instance(&bad_surface, timing())
        .err()
        .unwrap();
    assert!(err.message().starts_with("InvalidSurface:"), "{err}");

    let mut instance = engine.catalogue_instance(&edge(), timing()).unwrap();
    let err = instance.render_into(8, 8, &mut [0u8; 16]).err().unwrap();
    assert!(err.message().starts_with("InvalidSize:"), "{err}");
}

#[test]
fn a_catalogue_scene_round_trips_into_an_instance() {
    let json = water_ui_android::host::catalogue_scene_json(&edge()).unwrap();
    assert!(json.contains("\"version\""), "not a scene document");
    let Some(engine) = engine() else {
        eprintln!("skipped: no hardware GPU");
        return;
    };
    let mut instance = engine.scene_instance(&json, timing()).unwrap();
    instance.finish_immediately().unwrap();
    assert!(instance.is_finished());
}
