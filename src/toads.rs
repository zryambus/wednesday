use rand::seq::SliceRandom;

const TOADS: &[&str] = &[
    "9K4-jllrPrE",
    "bbat6cvgEJ8",
    "Oct2xKMGOno",
    "DREDJ4fkz-g",
    "gxm5SwfkwcI",
    "oVxFk_IIB2o",
    "SePVlroq6AI",
    "JHO61_wDC30",
    "EBNEPil4da0",
    "pXv4zQ6dYPQ",
    "hzGQSlrB1_o",
    "Y_xlWdgi1ew",
    "szqNmefKXxc",
    "OzQ-KvxLVT0",
    "zl6phK1mXC4",
    "7aTtNNjIyi4",
    "1CH-7qjz4D4",
    "YSDAAh6Lps4",
    "fyJGKEswuSc",
    "csqJK8wwaHw",
    "KSwnFzlPEuY",
    "aew9WTLqjDc",
    "m2Z0CyuyfMI",
    "VaPMUACYWww",
    "_87k7gxeVsw",
    "3RSL5k3yZOM",
    "VXc47lVx7Eo",
    "0W51GIxnwKc",
    "VfaNCw2bF48",
    "It8RbsGIe48",
    "NBPlPowAsNc",
    "IaE0g3oVIZ0",
    "VzigPnZ8OYE",
    "meuYC7FP7HU",
    "N3e7G9OxfhI",
    "IR0QUwGmo4A",
    "ESNBnxtpKqI",
    "036ItQLi-sQ",
    "Kz26jod9-cQ",
    "LrleLDD8CJM",
    "ZHS5yAwApUs",
    "PE8GlPpuLuY",
    "4Sr5pRpDZMk",
    "qCsYa8PeVfU",
    "-R40VcLKyIw",
    "7dr2s59XnBE",
    "iTl1l3GFMJ8",
    "In9Bs1wiF5s",
    "zHpFuOlPrlQ",
    "Xf_wuAQ-t44",
    "frNFBv2QIoE",
    "PAnKl7862qc",
];

pub fn get_toad() -> String {
    let video_id = TOADS.choose(&mut rand::thread_rng()).unwrap();
    format!("https://youtu.be/{}", video_id)
}
