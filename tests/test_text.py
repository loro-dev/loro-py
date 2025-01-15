from loro import ExportMode, LoroDoc, LoroMap, LoroText, StyleConfigMap, TextDelta, ValueOrContainer

class Wrapper:

    def __init__(self, doc: LoroDoc):
        self.doc = doc

    @classmethod
    def from_snapshot(cls, snapshot: bytes) -> 'Wrapper':
        doc = LoroDoc()
        doc.import_(snapshot)
        return cls(doc)

    def export(self) -> bytes:
        return self.doc.export(ExportMode.Snapshot())


def test_text_get_value():
    doc = LoroDoc()
    doc.config_text_style(StyleConfigMap.default_rich_text_config())
    text = doc.get_text("text")
    text.insert(0, "Hello world!")
    text.mark(start=0, end=5, key="bold", value=True)

    values = text.get_richtext_value()
    for index, delta in enumerate(values):
        if index == 0:
            assert "insert" in delta
            assert delta["insert"] == "Hello"
            assert delta["attributes"] == {"bold": True}
        elif index == 1:
            assert "insert" in delta
            assert delta["insert"] == " world!"
            assert "attributes" not in delta

def test_text_to_delta():
    doc = LoroDoc()
    doc.config_text_style(StyleConfigMap.default_rich_text_config())
    text = doc.get_text("text")
    text.insert(0, "Hello world!")
    text.mark(start=0, end=5, key="bold", value=True)

    deltas = text.to_delta()
    for index, delta in enumerate(deltas):
        if index == 0:
            assert isinstance(delta, TextDelta.Insert)
            assert delta.insert == "Hello"
            assert delta.attributes == {"bold": True}
        elif index == 1:
            assert isinstance(delta, TextDelta.Insert)
            assert delta.insert == " world!"
            assert delta.attributes == None

def test_wrapper():
    wrapper = Wrapper(LoroDoc())
    text = LoroText()
    text.insert(0, 'Hello world!')
    wrapper.doc.get_map('fields').insert_container('text', text)

    Wrapper.from_snapshot(wrapper.export()).doc.get_map('fields').get_deep_value()