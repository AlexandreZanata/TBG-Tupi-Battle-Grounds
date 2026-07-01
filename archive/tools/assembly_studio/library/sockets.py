"""Socket bone table for modular part attachment."""

from __future__ import annotations

SOCKET_BONES: dict[str, str] = {
    "head": "TBG_Head",
    "head_top": "TBG_Head",
    "spine_chest": "TBG_Spine1",
    "hand_r": "TBG_Hand.R",
    "hand_l": "TBG_Hand.L",
    "back": "TBG_Spine1",
}

CATEGORY_SOCKETS: dict[str, str] = {
    "heads": "TBG_Head",
    "hair": "TBG_Head",
    "hats": "TBG_Head",
    "noses": "TBG_Head",
    "beards": "TBG_Head",
    "armor": "TBG_Spine1",
    "backpacks": "TBG_Spine1",
    "weapons": "TBG_Hand.R",
    "wings": "TBG_Spine1",
    "tails": "TBG_Spine",
}

BASE_IDS = ("male", "female", "small", "large")
SHOWCASE_TRI_CAP = 12_000
