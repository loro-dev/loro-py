from loro import LoroDoc, PositionType, TextDelta


def test_utf16_insert_and_slice():
    doc = LoroDoc()
    text = doc.get_text("text")
    text.insert(0, "A😀C")

    emoji_utf16_pos = text.convert_pos(2, PositionType.Unicode, PositionType.Utf16)
    assert emoji_utf16_pos == 3
    assert text.convert_pos(emoji_utf16_pos, PositionType.Utf16, PositionType.Unicode) == 2

    text.insert_utf16(emoji_utf16_pos, "B")
    assert text.to_string() == "A😀BC"

    # Slice using UTF-16 indices around the emoji
    assert text.slice_utf16(1, 3) == "😀"

def test_slice_delta_and_marks_with_positions():
    doc = LoroDoc()
    text = doc.get_text("text")
    text.insert(0, "A😀BC")
    text.mark_utf16(0, 2, "bold", True)  # covers A and emoji

    delta = text.slice_delta(1, 3, PositionType.Unicode)
    assert isinstance(delta[0], TextDelta.Insert)
    assert delta[0].insert == "😀"
    assert delta[0].attributes == {"bold": True}
    assert isinstance(delta[1], TextDelta.Insert)
    assert delta[1].insert == "B"
    assert delta[1].attributes is None

    text.unmark_utf16(0, 2, "bold")
    delta_after = text.slice_delta(0, 4, PositionType.Unicode)
    for seg in delta_after:
        attrs = getattr(seg, "attributes", None)
        if attrs:
            assert "bold" not in attrs or not attrs["bold"]


def test_get_container_returns_handler():
    doc = LoroDoc()
    text = doc.get_text("text")
    container_id = text.id
    container = doc.get_container(container_id)
    assert container is not None
