import json

from loro import ExportMode, LoroDoc, VersionVector


def test_pending_updates_survive_a_snapshot_checkpoint():
    source = LoroDoc()
    source.get_text("text").insert(0, "base")
    base = source.export(ExportMode.Snapshot())
    before_a = source.oplog_vv
    source.get_text("text").insert(4, "A")
    a = source.export(ExportMode.Updates(before_a))
    after_a = source.oplog_vv
    source.get_text("text").insert(5, "B")
    b = source.export(ExportMode.Updates(after_a))

    receiver = LoroDoc()
    receiver.import_(base)
    receiver.import_(b)
    assert receiver.get_text("text").to_string() == "base"
    snapshot = receiver.export(ExportMode.Snapshot())
    pending = receiver.export_pending_updates()
    assert isinstance(pending, bytes)
    assert receiver.export_pending_updates() == pending

    restored = LoroDoc()
    restored.import_(snapshot)
    restored.import_(pending)
    assert restored.get_text("text").to_string() == "base"
    restored.import_(a)
    assert restored.get_text("text").to_string() == "baseAB"
    assert restored.export_pending_updates() is None


def test_pending_export_does_not_commit_local_edits():
    doc = LoroDoc()
    doc.get_text("text").insert(0, "local")
    pending_count = doc.get_pending_txn_len()
    assert pending_count > 0
    assert doc.export_pending_updates() is None
    assert doc.get_pending_txn_len() == pending_count


def test_json_export_keeps_the_existing_schema():
    doc = LoroDoc()
    doc.get_text("text").insert(0, "hello")
    doc.commit()
    updates = doc.export_json_updates(VersionVector(), doc.oplog_vv)
    assert "schema_version" in json.loads(updates)
    restored = LoroDoc()
    restored.import_json_updates(updates)
    assert restored.get_text("text").to_string() == "hello"
