from __future__ import annotations

import sys
from pathlib import Path

SIDECAR_ROOT = Path(__file__).resolve().parents[1]
if str(SIDECAR_ROOT) not in sys.path:
    sys.path.insert(0, str(SIDECAR_ROOT))

from platforms.douyin.mapper import aweme_to_media_item


def test_aweme_to_media_item_falls_back_to_origin_cover_when_cover_has_no_url() -> None:
    item = aweme_to_media_item(
        {
            "aweme_id": "7448579776181017897",
            "desc": "封面回退测试",
            "create_time": 1_788_881_600,
            "author": {"uid": "author-1", "nickname": "测试作者"},
            "video": {
                "duration": 12_000,
                "cover": {"uri": "cover-without-url-list"},
                "origin_cover": {
                    "url_list": ["//p9-sign.douyinpic.com/origin-cover.jpeg"]
                },
            },
        }
    )

    assert item["coverUrl"] == "https://p9-sign.douyinpic.com/origin-cover.jpeg"


def test_aweme_to_media_item_uses_image_post_info_for_gallery_cover() -> None:
    item = aweme_to_media_item(
        {
            "aweme_id": "7448579776181017898",
            "desc": "图集封面测试",
            "author": {"uid": "author-2", "nickname": "测试作者二"},
            "image_post_info": {
                "images": [
                    {"display_image": {"url_list": ["https://p3.douyinpic.com/gallery.webp"]}}
                ]
            },
        }
    )

    assert item["coverUrl"] == "https://p3.douyinpic.com/gallery.webp"


def test_aweme_to_media_item_prefers_non_p3_cover_mirror() -> None:
    item = aweme_to_media_item(
        {
            "aweme_id": "7448579776181017899",
            "desc": "封面镜像测试",
            "author": {"uid": "author-3", "nickname": "测试作者三"},
            "video": {
                "cover": {
                    "url_list": [
                        "https://p3-sign.douyinpic.com/cover.jpeg",
                        "https://p9-sign.douyinpic.com/cover.jpeg",
                    ]
                }
            },
        }
    )

    assert item["coverUrl"] == "https://p9-sign.douyinpic.com/cover.jpeg"
