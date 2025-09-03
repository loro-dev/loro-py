from loro import LoroDoc, LoroList

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
