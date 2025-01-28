use anyhow::Result;
use tempfile::tempdir;

use crate::common::{
    self, count_files, setup_generation_link_from_toplevel,
};

/// Install two generations that point at the same toplevel.
/// This should install two lanzaboote images and one kernel and one initrd.
#[test]
fn build_works() -> Result<()> {
    let esp = tempdir()?;
    let tmpdir = tempdir()?;
    let profiles = tempdir()?;
    let toplevel = common::setup_toplevel(tmpdir.path())?;

    let generation = setup_generation_link_from_toplevel(&toplevel, profiles.path(), 1)?;

    let stub_count = || count_files(&esp.path().join("EFI/Linux")).unwrap();
    let kernel_and_initrd_count = || count_files(&esp.path().join("EFI/nixos")).unwrap();

    let output1 = common::lanzaboote_build(esp.path(), generation)?;
    assert!(output1.status.success());
    assert_eq!(stub_count(), 1, "Wrong number of stubs after installation");
    assert_eq!(
        kernel_and_initrd_count(),
        2,
        "Wrong number of kernels & initrds after installation"
    );
    Ok(())
}
