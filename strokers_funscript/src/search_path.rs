use std::collections::BTreeMap;

use strokers_core::AxisKind;

pub const EXTENSIONS_TO_AXIS_KINDS: &[(&str, AxisKind)] = &[
    (".surge", AxisKind::Surge),
    (".sway", AxisKind::Sway),
    (".twist", AxisKind::Twist),
    (".roll", AxisKind::Roll),
    (".pitch", AxisKind::Pitch),
    // TODO others.
];

/// All discovered funscripts related to a given video.
/// There is a 'main' cluster and possibly one or more 'override' clusters,
/// letting you switch in alternative funscripts at will.
#[derive(Clone, Debug)]
pub struct FunscriptScan {
    pub main: FunscriptCluster,
    pub overrides: BTreeMap<String, FunscriptCluster>,
}

/// A cluster of funscript files, one per axis.
#[derive(Clone, Debug, Default)]
pub struct FunscriptCluster {
    pub scripts: BTreeMap<AxisKind, String>,
}

/// Given a list of filenames in the directory alongside the video,
/// search for funscripts that likely match the video.
pub fn scan_for_funscripts(
    dir_listing_of_files: &Vec<String>,
    scan_filename: &str,
) -> eyre::Result<FunscriptScan> {
    let scan_filename_without_extension = scan_filename
        .rsplit_once('.')
        .map(|(a, _)| a)
        .unwrap_or(scan_filename);

    let mut scan = FunscriptScan {
        main: Default::default(),
        overrides: Default::default(),
    };

    for file in dir_listing_of_files {
        let Some(unextended) = file.strip_prefix(scan_filename_without_extension) else {
            continue;
        };

        let Some(mut unextended) = unextended.strip_suffix(".funscript") else {
            continue;
        };

        let mut axis = AxisKind::Stroke;

        for (axis_suffix, axis_kind) in EXTENSIONS_TO_AXIS_KINDS {
            if let Some(new_unextended) = unextended.strip_suffix(axis_suffix) {
                axis = *axis_kind;
                unextended = new_unextended;
            }
        }

        let cluster_to_add_to = if unextended.is_empty() {
            &mut scan.main
        } else {
            scan.overrides.entry(unextended.to_owned()).or_default()
        };

        cluster_to_add_to.scripts.insert(axis, file.clone());
    }

    Ok(scan)
}
