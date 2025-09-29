import pytest

from loro import LoroDoc, LoroText


def make_text(content: str = "hello"):
    doc = LoroDoc()
    text = doc.get_text("text")
    text.insert(0, content)
    return doc, text


def test_lorotext_magic_methods_basic():
    _, text = make_text("hello")

    assert str(text) == "hello"
    assert repr(text) == 'LoroText("hello")'
    assert len(text) == 5
    assert text[0] == "h"
    assert text[-1] == "o"
    assert text[1:4] == "ell"


def test_lorotext_magic_methods_contains_and_add():
    doc, text = make_text("hello world")

    other = doc.get_text("other")
    other.insert(0, "world")

    assert "hello" in text
    assert other in text
    assert "absent" not in text

    assert text + "!" == "hello world!"
    assert text + other == "hello worldworld"
    assert "say " + text == "say hello world"
    assert other + text == "worldhello world"


def test_lorotext_getitem_type_errors():
    _, text = make_text("hello")

    with pytest.raises(TypeError):
        _ = text["invalid"]


def test_lorotree_contains():
    doc = LoroDoc()
    tree = doc.get_tree("tree")
    root = tree.create()
    child = tree.create(root)

    assert root in tree
    assert child in tree

    other_doc = LoroDoc()
    other_tree = other_doc.get_tree("other")
    other_root = other_tree.create()

    assert other_root not in tree


def test_lorocounter_numeric_magic_methods():
    doc = LoroDoc()
    counter = doc.get_counter("counter")
    counter.increment(10)
    doc.commit()

    assert float(counter) == pytest.approx(10.0)
    assert int(counter) == 10
    assert counter + 2 == pytest.approx(12.0)
    assert 2 + counter == pytest.approx(12.0)
    assert counter - 3 == pytest.approx(7.0)
    assert 3 - counter == pytest.approx(-7.0)

    counter.increment(5)
    doc.commit()
    assert counter.value == pytest.approx(15.0)

    counter.decrement(2)
    doc.commit()
    assert counter.value == pytest.approx(13.0)

    other = doc.get_counter("other")
    other.increment(4)
    doc.commit()

    assert counter + other == pytest.approx(17.0)
    assert -counter == pytest.approx(-13.0)
    assert abs(counter) == pytest.approx(13.0)


def test_lorolist_magic_methods():
    doc = LoroDoc()
    lst = doc.get_list("items")
    for i, value in enumerate([1, 2, "three"]):
        lst.insert(i, value)
    doc.commit()

    assert len(lst) == 3
    assert lst[0].value == 1
    assert [x.value for x in lst[1:3]] == [2, "three"]

    lst[1] = 42
    assert lst.to_vec() == [1, 42, "three"]

    del lst[0]
    assert lst.to_vec() == [42, "three"]

    lst[0:1] = ["alpha", "beta"]
    assert lst.to_vec() == ["alpha", "beta", "three"]

    lst[::2] = ["first", "second"]
    assert lst.to_vec() == ["first", "beta", "second"]

    with pytest.raises(ValueError):
        lst[::2] = ["only one"]

    del lst[1:3]
    assert lst.to_vec() == ["first"]

    lst[:] = ["reset", "list"]
    assert lst.to_vec() == ["reset", "list"]

    del lst[::-1]
    assert lst.to_vec() == []


def test_loromovable_list_magic_methods():
    doc = LoroDoc()
    mlist = doc.get_movable_list("items")
    for i, value in enumerate(["a", "b", "c"]):
        mlist.insert(i, value)

    assert len(mlist) == 3
    assert mlist[2].value == "c"
    assert [x.value for x in mlist[0:2]] == ["a", "b"]

    mlist[1] = "beta"
    assert mlist.to_vec() == ["a", "beta", "c"]

    del mlist[2]
    assert mlist.to_vec() == ["a", "beta"]

    mlist[:] = ["x", "y", "z"]
    assert mlist.to_vec() == ["x", "y", "z"]

    mlist[1:3] = ["Y", "Z", "extra"]
    assert mlist.to_vec() == ["x", "Y", "Z", "extra"]

    mlist[::2] = ["even", "odd"]
    assert mlist.to_vec() == ["even", "Y", "odd", "extra"]

    with pytest.raises(ValueError):
        mlist[::2] = ["mismatch"]

    del mlist[1::2]
    assert mlist.to_vec() == ["even", "odd"]

    del mlist[::-1]
    assert mlist.to_vec() == []


def test_loromap_magic_methods():
    doc = LoroDoc()
    map_obj = doc.get_map("map")

    map_obj["x"] = 10
    map_obj["y"] = "value"

    assert len(map_obj) == 2
    assert "x" in map_obj
    assert map_obj["x"].value == 10

    del map_obj["x"]
    assert "x" not in map_obj

    with pytest.raises(KeyError):
        _ = map_obj["missing"]

    with pytest.raises(KeyError):
        del map_obj["missing"]
