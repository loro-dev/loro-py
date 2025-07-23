from loro import LoroDoc, UndoManager


def test_undo_manager_group_basic():
    """Test basic group_start and group_end functionality"""
    doc = LoroDoc()
    undo_manager = UndoManager(doc)
    text = doc.get_text("text")

    # Start a group
    undo_manager.group_start()

    # Make multiple changes within the group
    text.update("hello")
    doc.commit()

    text.update("world")
    doc.commit()

    # End the group
    undo_manager.group_end()

    # Undo should revert all changes in the group
    undo_manager.undo()

    # Text should be empty after undo
    assert text.to_string() == ""


def test_undo_manager_group_multiple_operations():
    """Test group functionality with multiple text operations"""
    doc = LoroDoc()
    undo_manager = UndoManager(doc)
    text = doc.get_text("text")

    # Initial state
    text.update("initial")
    doc.commit()

    # Start a group
    undo_manager.group_start()

    # Multiple operations within the group
    text.update("hello")
    doc.commit()

    text.update("world")
    doc.commit()

    text.update("test")
    doc.commit()

    # End the group
    undo_manager.group_end()

    # Verify current state
    assert text.to_string() == "test"

    # Undo should revert to initial state
    undo_manager.undo()
    assert text.to_string() == "initial"


def test_undo_manager_group_with_insert():
    """Test group functionality with insert operations"""
    doc = LoroDoc()
    undo_manager = UndoManager(doc)
    text = doc.get_text("text")

    # Start a group
    undo_manager.group_start()

    # Use insert operations instead of update
    text.insert(0, "hello")
    doc.commit()

    text.insert(5, " world")
    doc.commit()

    # End the group
    undo_manager.group_end()

    # Verify current state
    assert text.to_string() == "hello world"

    # Undo should revert all changes
    undo_manager.undo()
    assert text.to_string() == ""


def test_undo_manager_group_redo():
    """Test redo functionality with groups"""
    doc = LoroDoc()
    undo_manager = UndoManager(doc)
    text = doc.get_text("text")

    # Start a group
    undo_manager.group_start()

    text.update("hello")
    doc.commit()

    text.update("world")
    doc.commit()

    # End the group
    undo_manager.group_end()

    # Verify initial state
    assert text.to_string() == "world"

    # Undo the group
    undo_manager.undo()
    assert text.to_string() == ""

    # Redo the group
    undo_manager.redo()
    assert text.to_string() == "world"


def test_undo_manager_group_without_commit():
    """Test group functionality without explicit commits"""
    doc = LoroDoc()
    undo_manager = UndoManager(doc)
    text = doc.get_text("text")

    # Start a group
    undo_manager.group_start()

    # Make changes without explicit commits
    text.update("hello")
    text.update("world")

    # End the group
    undo_manager.group_end()

    # Verify current state
    assert text.to_string() == "world"

    # Undo should revert all changes
    undo_manager.undo()
    assert text.to_string() == ""

def test_undo_manager_group_undo_count():
    """Test undo count with groups"""
    doc = LoroDoc()
    undo_manager = UndoManager(doc)
    text = doc.get_text("text")

    # Initial state
    text.update("initial")
    doc.commit()

    # Start a group
    undo_manager.group_start()

    text.update("hello")
    doc.commit()

    text.update("world")
    doc.commit()

    # End the group
    undo_manager.group_end()

    # Should have 2 undo steps: the group and the initial change
    assert undo_manager.undo_count() == 2

    # Undo the group
    undo_manager.undo()
    assert text.to_string() == "initial"
    assert undo_manager.undo_count() == 1

    # Undo the initial change
    undo_manager.undo()
    assert text.to_string() == ""
    assert undo_manager.undo_count() == 0 