fn main() -> anyhow::Result<()> {
    compile_gresources()?;
    #[cfg(windows)]
    compile_relm4_icons();
    compile_icon_winres()?;
    Ok(())
}
fn compile_relm4_icons() {
    relm4_icons_build::bundle_icons(
        // Name of the file that will be generated at `OUT_DIR`
        "icon_names.rs",
        // Optional app ID
        Some("github.linruohan.mytool"),
        // Custom base resource path:
        // * defaults to `/com/example/myapp` in this case if not specified explicitly
        // * or `/org/relm4` if app ID was not specified either
        None::<&str>,
        // Directory with custom icons (if any)
        None::<&str>,
        // List of icons to include
        ["ssd", "size-horizontally", "cross"],
    );
}

fn compile_icon_winres() -> anyhow::Result<()> {
    use anyhow::Context;
    let mut res = winresource::WindowsResource::new();
    res.set("OriginalFileName", "mytool.exe");
    res.set_icon("./data/icons/mytool.ico");
    res.compile()
        .context("Failed to compile winresource resource")
}
fn compile_gresources() -> anyhow::Result<()> {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "mytool.gresource",
    );
    Ok(())
}
