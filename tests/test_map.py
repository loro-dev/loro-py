import pytest

from loro import ExportMode, LoroDoc, LoroList

def test_map():
    doc = LoroDoc()
    map = doc.get_map("map")
    map.insert("key", "value")
    list = map.insert_container("key2", LoroList())
    list.insert(0, "value2")
    doc.commit()
    assert doc.get_deep_value() == {
        "map": {"key": "value", "key2": ["value2"]},
    }
    map["key2"] = "value2"
    assert map["key2"].value == "value2"

def test_ensure_mergeable_map_converges():
    doc1 = LoroDoc()
    doc1.peer_id = 1
    doc2 = LoroDoc()
    doc2.peer_id = 2

    profile1 = doc1.get_map("root").ensure_mergeable_map("profile")
    profile1.insert("name", "Ada")
    doc1.commit()

    profile2 = doc2.get_map("root").ensure_mergeable_map("profile")
    profile2.insert("age", 37)
    doc2.commit()

    doc1.import_(doc2.export(ExportMode.Snapshot()))
    doc2.import_(doc1.export(ExportMode.Snapshot()))

    expected = {"root": {"profile": {"name": "Ada", "age": 37}}}
    assert doc1.get_deep_value() == expected
    assert doc2.get_deep_value() == expected

def test_ensure_mergeable_rejects_occupied_key():
    doc = LoroDoc()
    root = doc.get_map("root")
    root.insert("profile", "occupied")

    with pytest.raises(BaseException):
        root.ensure_mergeable_map("profile")
