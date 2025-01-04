from loro import LoroDoc, VersionVector

def test_basic():
    doc = LoroDoc()
    text = doc.get_text("text")
    text.insert(0, "abc")
    doc.commit()
    assert text.to_string() == "abc"

def test_version_vector():
    doc1 = LoroDoc()
    assert doc1.state_vv == VersionVector()
    text = doc1.get_text("text")
    text.insert(0, "abc")

    snapshot = doc1.export({"mode": "snapshot"})
    doc2 = LoroDoc()
    assert doc1.state_vv != doc2.state_vv

    doc2.import_(snapshot)
    assert doc2.get_text("text").to_string() == "abc"
    assert doc1.state_vv == doc2.state_vv
