"""Regenerate Windows dependency notices from Cargo.lock and registry license files."""
import hashlib
import json
import re
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-msvc"


def cargo(*args):
    return subprocess.check_output(["cargo", *args], cwd=ROOT, text=True)


def main():
    metadata = json.loads(cargo("metadata", "--locked", "--format-version", "1", "--filter-platform", TARGET))
    tree = cargo("tree", "--locked", "--target", TARGET, "--edges", "normal,build", "--prefix", "none", "--format", "{p}")
    selected = {tuple(match.groups()) for line in tree.splitlines()
                if (match := re.match(r"([^ ]+) v([^ ]+)", line))}
    packages = sorted((p for p in metadata["packages"] if p["source"] and (p["name"], p["version"]) in selected),
                      key=lambda p: (p["name"], p["version"]))
    cache = {}
    texts = {}
    entries = []
    for package in packages:
        root = Path(package["manifest_path"]).parent
        files = sorted(p for p in root.rglob("*") if p.is_file()
                       and p.name.lower().startswith(("license", "licence", "copying", "copyright", "notice")))
        if package["name"] == "epaint_default_fonts":
            files = sorted(set(files) | set((root / "fonts").glob("*.txt")))
        documents = [(str(path.relative_to(root)), path.read_text(encoding="utf-8")) for path in files]
        # Some workspace crates omit the repository-level licenses from their package.
        # Use the exact source revision recorded in the published crate, not HEAD.
        if not any(p.parent == root for p in files):
            vcs = json.loads((root / ".cargo_vcs_info.json").read_text(encoding="utf-8"))
            repo = re.match(r"https://github.com/([^/]+/[^/]+)", package["repository"] or "")
            if not repo:
                raise RuntimeError(f"License source requires review: {package['name']}")
            base = f"https://raw.githubusercontent.com/{repo[1].removesuffix('.git')}/{vcs['git']['sha1']}"
            found = False
            for name in ("LICENSE", "LICENSE-MIT", "LICENSE-APACHE"):
                url = f"{base}/{name}"
                if url not in cache:
                    try:
                        with urllib.request.urlopen(url, timeout=30) as response:
                            cache[url] = response.read().decode("utf-8")
                    except urllib.error.HTTPError as error:
                        if error.code != 404:
                            raise
                        cache[url] = None
                if cache[url]:
                    documents.append((url, cache[url]))
                    found = True
            if not found:
                raise RuntimeError(f"Missing repository license: {package['name']}")
        if not documents or not package["license"]:
            raise RuntimeError(f"Missing license metadata: {package['name']}")
        references = []
        for source, text in documents:
            text = text.replace("\r\n", "\n").strip() + "\n"
            digest = hashlib.sha256(text.encode()).hexdigest()[:16]
            texts[digest] = text
            references.append(f"  {source} -> {digest}")
        entries.append(f"{package['name']} {package['version']}\nLicense: {package['license']}\n"
                       f"Source: {package['repository'] or 'https://crates.io/crates/' + package['name']}\n"
                       + "\n".join(references))
    preamble = ("Third-party licenses for the Windows x64 dependency graph (normal and build dependencies).\n"
                "Generated from Cargo.lock by scripts/generate_notices.py.\n"
                "License expressions are upstream declarations; alternatives remain alternatives.\n"
                "Default font notices are included. System-installed Chinese fonts are not redistributed.\n\n")
    bundle = preamble + "\n\n".join(entries) + "\n\n"
    for digest, text in sorted(texts.items()):
        bundle += f"{'=' * 72}\nLicense text {digest}\n{'=' * 72}\n{text}\n"
    (ROOT / "THIRD_PARTY_LICENSES.txt").write_text(bundle.rstrip() + "\n", encoding="utf-8")
    rows = ["| Package | Version | Declared license |", "| --- | --- | --- |"]
    for p in packages:
        rows.append(f"| [{p['name']}](https://crates.io/crates/{p['name']}/{p['version']}) | {p['version']} | {p['license']} |")
    notice = ("# Third-party notices\n\n"
              "This inventory covers the Windows x64 normal and build dependency graph in `Cargo.lock`; "
              "it is not a claim that every listed build tool is embedded in the executable. "
              "Full available license and copyright notices, including egui's bundled fonts, "
              "are collected in [THIRD_PARTY_LICENSES.txt](THIRD_PARTY_LICENSES.txt).\n\n"
              "The application's MIT license does not replace these licenses. "
              "Windows system fonts are loaded from the operating system and are not distributed here.\n\n"
              "Mac Mouse Fix is a design reference, not a Cargo dependency. "
              "See the attribution and original license link in [README.md](README.md#许可证与致谢).\n\n"
              "After changing dependencies, run `python scripts/generate_notices.py` with Python 3.9+ "
              "and Rust installed. The script reads Cargo metadata and, when necessary, retrieves missing "
              "license files from the exact upstream Git revision recorded in each crate. "
              "Review newly introduced licenses before distributing a new build.\n\n"
              + "\n".join(rows) + "\n")
    (ROOT / "THIRD_PARTY_NOTICES.md").write_text(notice, encoding="utf-8")
    print(f"Collected {len(packages)} packages and {len(texts)} unique license texts.")


if __name__ == "__main__":
    main()
