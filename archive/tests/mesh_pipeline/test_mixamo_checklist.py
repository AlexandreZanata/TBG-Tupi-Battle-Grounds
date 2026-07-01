"""Contract: Mixamo fallback checklist references upload URL and import CLI."""

from tools.mesh_pipeline.mixamo_checklist import MIXAMO_UPLOAD_URL, mixamo_checklist_lines


def test_mixamo_checklist_includes_upload_url():
    text = "\n".join(mixamo_checklist_lines())
    assert MIXAMO_UPLOAD_URL in text
    assert "import_mixamo_fbx.py" in text


def test_mixamo_checklist_mentions_body_class_library():
    text = "\n".join(mixamo_checklist_lines())
    assert "body-class" in text.lower()
    assert "NOT Mixamo" in text
