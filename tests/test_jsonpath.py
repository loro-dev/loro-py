from loro import LoroDoc


def test_subscribe_jsonpath_triggers_once_on_change():
    doc = LoroDoc()
    users = doc.get_map("users")
    calls = {"count": 0}

    sub = doc.subscribe_jsonpath("$.users.alice", lambda: calls.__setitem__("count", calls["count"] + 1))

    users.insert("alice", 1)
    doc.commit()

    assert calls["count"] >= 1

    sub.unsubscribe()

    users.insert("alice", 2)
    doc.commit()

    # No extra callbacks after unsubscribe
    assert calls["count"] == 1
