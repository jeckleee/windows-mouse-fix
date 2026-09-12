# Third-party notices

This inventory covers the Windows x64 normal and build dependency graph in `Cargo.lock`; it is not a claim that every listed build tool is embedded in the executable. Full available license and copyright notices, including egui's bundled fonts, are collected in [THIRD_PARTY_LICENSES.txt](THIRD_PARTY_LICENSES.txt).

The application's MIT license does not replace these licenses. Windows system fonts are loaded from the operating system and are not distributed here.

Mac Mouse Fix is a design reference, not a Cargo dependency. See the attribution and original license link in [README.md](README.md#许可证与致谢).

After changing dependencies, run `python scripts/generate_notices.py` with Python 3.9+ and Rust installed. The script reads Cargo metadata and, when necessary, retrieves missing license files from the exact upstream Git revision recorded in each crate. Review newly introduced licenses before distributing a new build.

| Package | Version | Declared license |
| --- | --- | --- |
| [ab_glyph](https://crates.io/crates/ab_glyph/0.2.32) | 0.2.32 | Apache-2.0 |
| [ab_glyph_rasterizer](https://crates.io/crates/ab_glyph_rasterizer/0.1.10) | 0.1.10 | Apache-2.0 |
| [adler2](https://crates.io/crates/adler2/2.0.1) | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| [ahash](https://crates.io/crates/ahash/0.8.12) | 0.8.12 | MIT OR Apache-2.0 |
| [arboard](https://crates.io/crates/arboard/3.6.1) | 3.6.1 | MIT OR Apache-2.0 |
| [atomic-waker](https://crates.io/crates/atomic-waker/1.1.2) | 1.1.2 | Apache-2.0 OR MIT |
| [autocfg](https://crates.io/crates/autocfg/1.5.1) | 1.5.1 | Apache-2.0 OR MIT |
| [base64](https://crates.io/crates/base64/0.22.1) | 0.22.1 | MIT OR Apache-2.0 |
| [bitflags](https://crates.io/crates/bitflags/2.13.2) | 2.13.2 | MIT OR Apache-2.0 |
| [bytemuck](https://crates.io/crates/bytemuck/1.25.2) | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| [bytemuck_derive](https://crates.io/crates/bytemuck_derive/1.12.1) | 1.12.1 | Zlib OR Apache-2.0 OR MIT |
| [byteorder-lite](https://crates.io/crates/byteorder-lite/0.1.0) | 0.1.0 | Unlicense OR MIT |
| [bytes](https://crates.io/crates/bytes/1.12.1) | 1.12.1 | MIT |
| [cc](https://crates.io/crates/cc/1.4.5) | 1.4.5 | MIT OR Apache-2.0 |
| [cfg-if](https://crates.io/crates/cfg-if/1.0.4) | 1.0.4 | MIT OR Apache-2.0 |
| [cfg_aliases](https://crates.io/crates/cfg_aliases/0.2.2) | 0.2.2 | MIT |
| [clipboard-win](https://crates.io/crates/clipboard-win/5.4.1) | 5.4.1 | BSL-1.0 |
| [crc32fast](https://crates.io/crates/crc32fast/1.5.1) | 1.5.1 | MIT OR Apache-2.0 |
| [cursor-icon](https://crates.io/crates/cursor-icon/1.2.0) | 1.2.0 | MIT OR Apache-2.0 OR Zlib |
| [displaydoc](https://crates.io/crates/displaydoc/0.2.7) | 0.2.7 | MIT OR Apache-2.0 |
| [document-features](https://crates.io/crates/document-features/0.2.12) | 0.2.12 | MIT OR Apache-2.0 |
| [dpi](https://crates.io/crates/dpi/0.1.2) | 0.1.2 | Apache-2.0 AND MIT |
| [ecolor](https://crates.io/crates/ecolor/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [eframe](https://crates.io/crates/eframe/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [egui](https://crates.io/crates/egui/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [egui-winit](https://crates.io/crates/egui-winit/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [egui_glow](https://crates.io/crates/egui_glow/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [emath](https://crates.io/crates/emath/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [epaint](https://crates.io/crates/epaint/0.33.3) | 0.33.3 | MIT OR Apache-2.0 |
| [epaint_default_fonts](https://crates.io/crates/epaint_default_fonts/0.33.3) | 0.33.3 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 |
| [error-code](https://crates.io/crates/error-code/3.4.0) | 3.4.0 | BSL-1.0 |
| [fdeflate](https://crates.io/crates/fdeflate/0.3.7) | 0.3.7 | MIT OR Apache-2.0 |
| [find-msvc-tools](https://crates.io/crates/find-msvc-tools/0.1.12) | 0.1.12 | MIT OR Apache-2.0 |
| [flate2](https://crates.io/crates/flate2/1.1.10) | 1.1.10 | MIT OR Apache-2.0 |
| [form_urlencoded](https://crates.io/crates/form_urlencoded/1.2.2) | 1.2.2 | MIT OR Apache-2.0 |
| [futures-channel](https://crates.io/crates/futures-channel/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-core](https://crates.io/crates/futures-core/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-io](https://crates.io/crates/futures-io/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-sink](https://crates.io/crates/futures-sink/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-task](https://crates.io/crates/futures-task/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| [futures-util](https://crates.io/crates/futures-util/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| [getrandom](https://crates.io/crates/getrandom/0.2.17) | 0.2.17 | MIT OR Apache-2.0 |
| [gl_generator](https://crates.io/crates/gl_generator/0.14.0) | 0.14.0 | Apache-2.0 |
| [glow](https://crates.io/crates/glow/0.16.0) | 0.16.0 | MIT OR Apache-2.0 OR Zlib |
| [glutin](https://crates.io/crates/glutin/0.32.3) | 0.32.3 | Apache-2.0 |
| [glutin-winit](https://crates.io/crates/glutin-winit/0.5.0) | 0.5.0 | MIT |
| [glutin_egl_sys](https://crates.io/crates/glutin_egl_sys/0.7.1) | 0.7.1 | Apache-2.0 |
| [glutin_wgl_sys](https://crates.io/crates/glutin_wgl_sys/0.6.1) | 0.6.1 | Apache-2.0 |
| [http](https://crates.io/crates/http/1.5.0) | 1.5.0 | MIT OR Apache-2.0 |
| [http-body](https://crates.io/crates/http-body/1.1.0) | 1.1.0 | MIT |
| [http-body-util](https://crates.io/crates/http-body-util/0.1.5) | 0.1.5 | MIT |
| [httparse](https://crates.io/crates/httparse/1.10.1) | 1.10.1 | MIT OR Apache-2.0 |
| [hyper](https://crates.io/crates/hyper/1.11.1) | 1.11.1 | MIT |
| [hyper-rustls](https://crates.io/crates/hyper-rustls/0.27.9) | 0.27.9 | Apache-2.0 OR ISC OR MIT |
| [hyper-util](https://crates.io/crates/hyper-util/0.1.20) | 0.1.20 | MIT |
| [icu_collections](https://crates.io/crates/icu_collections/2.3.0) | 2.3.0 | Unicode-3.0 |
| [icu_locale_core](https://crates.io/crates/icu_locale_core/2.3.0) | 2.3.0 | Unicode-3.0 |
| [icu_normalizer](https://crates.io/crates/icu_normalizer/2.3.0) | 2.3.0 | Unicode-3.0 |
| [icu_normalizer_data](https://crates.io/crates/icu_normalizer_data/2.3.0) | 2.3.0 | Unicode-3.0 |
| [icu_properties](https://crates.io/crates/icu_properties/2.3.0) | 2.3.0 | Unicode-3.0 |
| [icu_properties_data](https://crates.io/crates/icu_properties_data/2.3.0) | 2.3.0 | Unicode-3.0 |
| [icu_provider](https://crates.io/crates/icu_provider/2.3.1) | 2.3.1 | Unicode-3.0 |
| [idna](https://crates.io/crates/idna/1.1.0) | 1.1.0 | MIT OR Apache-2.0 |
| [idna_adapter](https://crates.io/crates/idna_adapter/1.2.2) | 1.2.2 | Apache-2.0 OR MIT |
| [image](https://crates.io/crates/image/0.25.10) | 0.25.10 | MIT OR Apache-2.0 |
| [ipnet](https://crates.io/crates/ipnet/2.12.2) | 2.12.2 | MIT OR Apache-2.0 |
| [itoa](https://crates.io/crates/itoa/1.0.18) | 1.0.18 | MIT OR Apache-2.0 |
| [khronos_api](https://crates.io/crates/khronos_api/3.1.0) | 3.1.0 | Apache-2.0 |
| [libc](https://crates.io/crates/libc/0.2.189) | 0.2.189 | MIT OR Apache-2.0 |
| [libloading](https://crates.io/crates/libloading/0.8.9) | 0.8.9 | ISC |
| [litemap](https://crates.io/crates/litemap/0.8.3) | 0.8.3 | Unicode-3.0 |
| [litrs](https://crates.io/crates/litrs/1.0.0) | 1.0.0 | MIT OR Apache-2.0 |
| [lock_api](https://crates.io/crates/lock_api/0.4.14) | 0.4.14 | MIT OR Apache-2.0 |
| [log](https://crates.io/crates/log/0.4.34) | 0.4.34 | MIT OR Apache-2.0 |
| [memchr](https://crates.io/crates/memchr/2.8.3) | 2.8.3 | Unlicense OR MIT |
| [memmap2](https://crates.io/crates/memmap2/0.9.11) | 0.9.11 | MIT OR Apache-2.0 |
| [memoffset](https://crates.io/crates/memoffset/0.9.1) | 0.9.1 | MIT |
| [miniz_oxide](https://crates.io/crates/miniz_oxide/0.8.9) | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| [miniz_oxide](https://crates.io/crates/miniz_oxide/0.9.1) | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| [mio](https://crates.io/crates/mio/1.2.3) | 1.2.3 | MIT |
| [moxcms](https://crates.io/crates/moxcms/0.8.1) | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| [nohash-hasher](https://crates.io/crates/nohash-hasher/0.2.0) | 0.2.0 | Apache-2.0 OR MIT |
| [num-traits](https://crates.io/crates/num-traits/0.2.19) | 0.2.19 | MIT OR Apache-2.0 |
| [once_cell](https://crates.io/crates/once_cell/1.21.4) | 1.21.4 | MIT OR Apache-2.0 |
| [owned_ttf_parser](https://crates.io/crates/owned_ttf_parser/0.25.1) | 0.25.1 | Apache-2.0 |
| [parking_lot](https://crates.io/crates/parking_lot/0.12.5) | 0.12.5 | MIT OR Apache-2.0 |
| [parking_lot_core](https://crates.io/crates/parking_lot_core/0.9.12) | 0.9.12 | MIT OR Apache-2.0 |
| [percent-encoding](https://crates.io/crates/percent-encoding/2.3.2) | 2.3.2 | MIT OR Apache-2.0 |
| [pin-project-lite](https://crates.io/crates/pin-project-lite/0.2.17) | 0.2.17 | Apache-2.0 OR MIT |
| [png](https://crates.io/crates/png/0.18.1) | 0.18.1 | MIT OR Apache-2.0 |
| [potential_utf](https://crates.io/crates/potential_utf/0.1.6) | 0.1.6 | Unicode-3.0 |
| [proc-macro2](https://crates.io/crates/proc-macro2/1.0.107) | 1.0.107 | MIT OR Apache-2.0 |
| [profiling](https://crates.io/crates/profiling/1.0.18) | 1.0.18 | MIT OR Apache-2.0 |
| [pxfm](https://crates.io/crates/pxfm/0.1.30) | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| [quote](https://crates.io/crates/quote/1.0.47) | 1.0.47 | MIT OR Apache-2.0 |
| [raw-window-handle](https://crates.io/crates/raw-window-handle/0.6.2) | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| [reqwest](https://crates.io/crates/reqwest/0.12.28) | 0.12.28 | MIT OR Apache-2.0 |
| [ring](https://crates.io/crates/ring/0.17.14) | 0.17.14 | Apache-2.0 AND ISC |
| [rustls](https://crates.io/crates/rustls/0.23.44) | 0.23.44 | Apache-2.0 OR ISC OR MIT |
| [rustls-pki-types](https://crates.io/crates/rustls-pki-types/1.15.1) | 1.15.1 | MIT OR Apache-2.0 |
| [rustls-webpki](https://crates.io/crates/rustls-webpki/0.103.15) | 0.103.15 | ISC |
| [ryu](https://crates.io/crates/ryu/1.0.23) | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| [scopeguard](https://crates.io/crates/scopeguard/1.2.0) | 1.2.0 | MIT OR Apache-2.0 |
| [semver](https://crates.io/crates/semver/1.0.28) | 1.0.28 | MIT OR Apache-2.0 |
| [serde](https://crates.io/crates/serde/1.0.229) | 1.0.229 | MIT OR Apache-2.0 |
| [serde_core](https://crates.io/crates/serde_core/1.0.229) | 1.0.229 | MIT OR Apache-2.0 |
| [serde_derive](https://crates.io/crates/serde_derive/1.0.229) | 1.0.229 | MIT OR Apache-2.0 |
| [serde_json](https://crates.io/crates/serde_json/1.0.151) | 1.0.151 | MIT OR Apache-2.0 |
| [serde_urlencoded](https://crates.io/crates/serde_urlencoded/0.7.1) | 0.7.1 | MIT/Apache-2.0 |
| [shlex](https://crates.io/crates/shlex/2.0.1) | 2.0.1 | MIT OR Apache-2.0 |
| [simd-adler32](https://crates.io/crates/simd-adler32/0.3.10) | 0.3.10 | MIT |
| [slab](https://crates.io/crates/slab/0.4.12) | 0.4.12 | MIT |
| [smallvec](https://crates.io/crates/smallvec/1.16.1) | 1.16.1 | MIT OR Apache-2.0 |
| [smol_str](https://crates.io/crates/smol_str/0.2.2) | 0.2.2 | MIT OR Apache-2.0 |
| [socket2](https://crates.io/crates/socket2/0.6.5) | 0.6.5 | MIT OR Apache-2.0 |
| [stable_deref_trait](https://crates.io/crates/stable_deref_trait/1.2.1) | 1.2.1 | MIT OR Apache-2.0 |
| [static_assertions](https://crates.io/crates/static_assertions/1.1.0) | 1.1.0 | MIT OR Apache-2.0 |
| [subtle](https://crates.io/crates/subtle/2.6.1) | 2.6.1 | BSD-3-Clause |
| [syn](https://crates.io/crates/syn/2.0.119) | 2.0.119 | MIT OR Apache-2.0 |
| [syn](https://crates.io/crates/syn/3.0.5) | 3.0.5 | MIT OR Apache-2.0 |
| [sync_wrapper](https://crates.io/crates/sync_wrapper/1.0.2) | 1.0.2 | Apache-2.0 |
| [synstructure](https://crates.io/crates/synstructure/0.13.2) | 0.13.2 | MIT |
| [tinystr](https://crates.io/crates/tinystr/0.8.4) | 0.8.4 | Unicode-3.0 |
| [tokio](https://crates.io/crates/tokio/1.53.1) | 1.53.1 | MIT |
| [tokio-rustls](https://crates.io/crates/tokio-rustls/0.26.5) | 0.26.5 | MIT OR Apache-2.0 |
| [tower](https://crates.io/crates/tower/0.5.3) | 0.5.3 | MIT |
| [tower-http](https://crates.io/crates/tower-http/0.6.11) | 0.6.11 | MIT |
| [tower-layer](https://crates.io/crates/tower-layer/0.3.3) | 0.3.3 | MIT |
| [tower-service](https://crates.io/crates/tower-service/0.3.3) | 0.3.3 | MIT |
| [tracing](https://crates.io/crates/tracing/0.1.44) | 0.1.44 | MIT |
| [tracing-core](https://crates.io/crates/tracing-core/0.1.36) | 0.1.36 | MIT |
| [try-lock](https://crates.io/crates/try-lock/0.2.5) | 0.2.5 | MIT |
| [ttf-parser](https://crates.io/crates/ttf-parser/0.25.1) | 0.25.1 | MIT OR Apache-2.0 |
| [unicode-ident](https://crates.io/crates/unicode-ident/1.0.24) | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| [unicode-segmentation](https://crates.io/crates/unicode-segmentation/1.13.3) | 1.13.3 | MIT OR Apache-2.0 |
| [untrusted](https://crates.io/crates/untrusted/0.9.0) | 0.9.0 | ISC |
| [url](https://crates.io/crates/url/2.5.8) | 2.5.8 | MIT OR Apache-2.0 |
| [utf8_iter](https://crates.io/crates/utf8_iter/1.0.4) | 1.0.4 | Apache-2.0 OR MIT |
| [version_check](https://crates.io/crates/version_check/0.9.5) | 0.9.5 | MIT/Apache-2.0 |
| [want](https://crates.io/crates/want/0.3.1) | 0.3.1 | MIT |
| [web-time](https://crates.io/crates/web-time/1.1.0) | 1.1.0 | MIT OR Apache-2.0 |
| [webbrowser](https://crates.io/crates/webbrowser/1.2.4) | 1.2.4 | MIT OR Apache-2.0 |
| [webpki-roots](https://crates.io/crates/webpki-roots/1.0.9) | 1.0.9 | CDLA-Permissive-2.0 |
| [windows-link](https://crates.io/crates/windows-link/0.2.1) | 0.2.1 | MIT OR Apache-2.0 |
| [windows-sys](https://crates.io/crates/windows-sys/0.52.0) | 0.52.0 | MIT OR Apache-2.0 |
| [windows-sys](https://crates.io/crates/windows-sys/0.60.2) | 0.60.2 | MIT OR Apache-2.0 |
| [windows-sys](https://crates.io/crates/windows-sys/0.61.2) | 0.61.2 | MIT OR Apache-2.0 |
| [windows-targets](https://crates.io/crates/windows-targets/0.52.6) | 0.52.6 | MIT OR Apache-2.0 |
| [windows-targets](https://crates.io/crates/windows-targets/0.53.5) | 0.53.5 | MIT OR Apache-2.0 |
| [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc/0.52.6) | 0.52.6 | MIT OR Apache-2.0 |
| [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc/0.53.1) | 0.53.1 | MIT OR Apache-2.0 |
| [winit](https://crates.io/crates/winit/0.30.13) | 0.30.13 | Apache-2.0 |
| [winresource](https://crates.io/crates/winresource/0.1.31) | 0.1.31 | MIT |
| [writeable](https://crates.io/crates/writeable/0.6.4) | 0.6.4 | Unicode-3.0 |
| [xml-rs](https://crates.io/crates/xml-rs/0.8.29) | 0.8.29 | MIT |
| [yoke](https://crates.io/crates/yoke/0.8.3) | 0.8.3 | Unicode-3.0 |
| [yoke-derive](https://crates.io/crates/yoke-derive/0.8.2) | 0.8.2 | Unicode-3.0 |
| [zerocopy](https://crates.io/crates/zerocopy/0.8.57) | 0.8.57 | BSD-2-Clause OR Apache-2.0 OR MIT |
| [zerofrom](https://crates.io/crates/zerofrom/0.1.8) | 0.1.8 | Unicode-3.0 |
| [zerofrom-derive](https://crates.io/crates/zerofrom-derive/0.1.7) | 0.1.7 | Unicode-3.0 |
| [zeroize](https://crates.io/crates/zeroize/1.9.0) | 1.9.0 | Apache-2.0 OR MIT |
| [zerotrie](https://crates.io/crates/zerotrie/0.2.5) | 0.2.5 | Unicode-3.0 |
| [zerovec](https://crates.io/crates/zerovec/0.11.8) | 0.11.8 | Unicode-3.0 |
| [zerovec-derive](https://crates.io/crates/zerovec-derive/0.11.6) | 0.11.6 | Unicode-3.0 |
| [zmij](https://crates.io/crates/zmij/1.0.23) | 1.0.23 | MIT |
