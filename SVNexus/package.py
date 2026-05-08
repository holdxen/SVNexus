import subprocess
import os
import platform
import shutil
import tempfile
from pathlib import Path
from typing import Callable


app_name = "SVNexus"

tmp = tempfile.TemporaryDirectory()

print(f"Package in: {tmp.name}")

subprocess.run(["csr", "--release"])

def package_linux():

    os.makedirs(Path(tmp.name) / "DEBIAN", exist_ok=True)

    files = f"{tmp.name}/opt/svnexus"

    os.makedirs(files, exist_ok=True)

    subprocess.run(["dotnet", "publish", "--sc", "-c", "Release", "-o", files])

    shutil.copy("rust/target/release/libengine.so", files)
    shutil.copy("Assets/svnexus-icon.svg", files)


    current = os.getcwd()

    os.chdir(f"{current}/rust/deps/linux-x64")

    subprocess.run(["tar", "-zcvf", f"{files}/svn.tar.gz", "./svn"])


    os.chdir(files)

    subprocess.run(["tar", "-zxvf", "svn.tar.gz"])

    subprocess.run(["rm", f"{files}/svn.tar.gz"])

    version = "0.0.1"


    with open(Path(tmp.name) / "DEBIAN" / "control", "w", encoding="utf-8") as f:
        control = f"""Package: {app_name}
Version: {version}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: holdxen zhousir429@gmail.com
Description: Subversion client with a modern interface and enhanced features.
"""
        f.write(control)

    os.makedirs(f"{tmp.name}/usr/share/applications", exist_ok=True)


    with open(f"{tmp.name}/usr/share/applications/svnexus.desktop", "w", encoding="utf-8") as f:
        entry = f"""[Desktop Entry]
Type=Application
Name={app_name}
Comment=Subversion client with a modern interface and enhanced features
Exec=/opt/svnexus/SVNexus
Icon=/opt/svnexus/svnexus-icon.svg
Terminal=false
Categories=Utility;
"""
        f.write(entry)
    os.chdir(current)

    subprocess.run(["dpkg-deb", "--build", tmp.name, f"svnexus_{version}_amd64.deb"])

def package_darwin():
    root = Path(tmp.name) / "root"
    contents = root / f"{app_name}.app" / "Contents"
    macos = contents / "MacOS"
    resources = contents / "Resources"

    os.makedirs(macos, exist_ok=True)
    os.makedirs(contents, exist_ok=True)
    os.makedirs(resources, exist_ok=True)


    subprocess.run(["dotnet", "publish", "--sc", "-c", "Release", "-o", macos])

    shutil.copy("rust/target/release/libengine.dylib", macos)

    svn = "./rust/deps/macos-aarch64"
    os.remove(f"{svn}/svn.tar.gz")
    subprocess.run(["tar", "zcvf", "svn.tar.gz", "./svn"], cwd=svn)
    shutil.copy(f"{svn}/svn.tar.gz", macos)

    subprocess.run(["tar", "zxvf", "svn.tar.gz"], cwd=macos)
    os.remove(macos / "svn.tar.gz")


    svg = tempfile.TemporaryDirectory()

    def generate(source: str, size: int, dest: str, two: bool):
        file = f"{dest}/icon_{size}x{size}.png"
        if two:
            file = f"{dest}/icon_{size//2}x{size//2}@2x.png"
        subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), source, "-o", file])

    iconset = f"{svg.name}/icon.iconset"
    os.makedirs(iconset)

    source = "Assets/svnexus-icon.svg"


    generate(source, 16, iconset, False)
    generate(source, 32, iconset, True)
    generate(source, 32, iconset, False)
    generate(source, 64, iconset, True)
    generate(source, 128, iconset, False)
    generate(source, 256, iconset, True)
    generate(source, 256, iconset, False)
    generate(source, 512, iconset, True)
    generate(source, 512, iconset, False)
    generate(source, 1024, iconset, True)


    subprocess.run(["iconutil", "-c", "icns", "icon.iconset", "-o", "AppIcon.icns"], cwd=svg.name)

    subprocess.run(["tree", "."], cwd=svg.name)

    shutil.copy(Path(svg.name) / "AppIcon.icns", resources)

    subprocess.run(["tree", "."], cwd=svg.name)


    info = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>{app_name}</string>

    <key>CFBundleDisplayName</key>
    <string>{app_name}</string>

    <key>CFBundleExecutable</key>
    <string>{app_name}</string>

    <key>CFBundleIdentifier</key>
    <string>io.github.holdxen.{app_name}</string>

    <key>CFBundlePackageType</key>
    <string>APPL</string>

    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>

    <key>CFBundleVersion</key>
    <string>1</string>

    <key>CFBundleIconFile</key>
    <string>AppIcon</string>

    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>

    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"""
    with open(contents / "Info.plist", "w", encoding="utf-8") as f:
        f.write(info)

    subprocess.run(["ln", "-s", "/Applications", "Applications"], cwd=root)

    subprocess.run(["hdiutil", "create", "-volname", app_name, "-srcfolder", "root", "-ov", "-format", "UDZO", f"{app_name}.dmg"], cwd=tmp.name)

    shutil.copy(f"{tmp.name}/{app_name}.dmg", ".")


os_name = platform.system()

if os_name == "Windows":
    print("当前是 Windows")
elif os_name == "Linux":
    print("当前是 Linux")
    package_linux()
elif os_name == "Darwin":
    print("当前是 macOS")
    package_darwin()
else:
    print("未知操作系统")

