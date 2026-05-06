import subprocess
import os
import platform
import shutil
import tempfile
from pathlib import Path


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
    macos = Path(tmp.name) / f"{app_name}.app" / "Contents" / "MacOS"
    os.makedirs(macos, exist_ok=True)
    subprocess.run(["dotnet", "publish", "--sc", "-c", "Release", "-o", macos])
    shutil.copy("rust/target/release/libengine.dylib", macos)
    shutil.copy("Assets/svnexus-icon.svg", macos)
    pass

os_name = platform.system()

if os_name == "Windows":
    print("当前是 Windows")
elif os_name == "Linux":
    print("当前是 Linux")
    package_linux()
elif os_name == "Darwin":
    print("当前是 macOS")
else:
    print("未知操作系统")

