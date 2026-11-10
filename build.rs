use image::{
    codecs::ico::{IcoEncoder, IcoFrame},
    ExtendedColorType, RgbaImage,
};
use std::{env, fs::File, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=assets/app-icon.svg");
    println!("cargo:rerun-if-changed=build.rs");
    let output = PathBuf::from(env::var("OUT_DIR")?);
    let svg = std::fs::read("assets/app-icon.svg")?;
    let tree = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default())?;
    let mut frames = Vec::new();
    for size in [16, 24, 32, 48, 64, 128, 256] {
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(size, size).ok_or("Icongroesse ungueltig")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(size as f32 / 256.0, size as f32 / 256.0),
            &mut pixmap.as_mut(),
        );
        let pixels: Vec<u8> = pixmap
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let color = pixel.demultiply();
                [color.red(), color.green(), color.blue(), color.alpha()]
            })
            .collect();
        let icon =
            RgbaImage::from_raw(size, size, pixels).ok_or("Icon konnte nicht erstellt werden")?;
        frames.push(IcoFrame::as_png(
            icon.as_raw(),
            size,
            size,
            ExtendedColorType::Rgba8,
        )?);
        if size == 256 {
            icon.save(output.join("app-icon.png"))?;
        }
    }
    let icon_path = output.join("app-icon.ico");
    IcoEncoder::new(File::create(&icon_path)?).encode_images(&frames)?;
    if env::var("CARGO_CFG_TARGET_OS")? == "windows" {
        let major: u16 = env::var("CARGO_PKG_VERSION_MAJOR")?.parse()?;
        let minor: u16 = env::var("CARGO_PKG_VERSION_MINOR")?.parse()?;
        let patch: u16 = env::var("CARGO_PKG_VERSION_PATCH")?.parse()?;
        let native_version =
            (u64::from(major) << 48) | (u64::from(minor) << 32) | (u64::from(patch) << 16);
        winresource::WindowsResource::new()
            .set_icon(icon_path.to_str().ok_or("Ungueltiger Iconpfad")?)
            .set("CompanyName", "OPTIMA Computer Service")
            .set("ProductName", "SchILD3 Updater")
            .set("FileDescription", "OPTIMA SchILD3 Updater")
                        .set("FileVersion", &env::var("CARGO_PKG_VERSION")?)
                        .set("ProductVersion", &env::var("CARGO_PKG_VERSION")?)
                        .set_version_info(winresource::VersionInfo::FILEVERSION, native_version)
                        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, native_version)
                        .set_manifest(&format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity version="{major}.{minor}.{patch}.0" processorArchitecture="*" name="OPTIMA.SchILD3Updater" type="win32"/>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3"><security><requestedPrivileges>
    <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
  </requestedPrivileges></security></trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1"><application>
    <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
  </application></compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3"><windowsSettings>
    <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
    <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
  </windowsSettings></application>
</assembly>"#))
            .compile()?;
    }
    Ok(())
}
