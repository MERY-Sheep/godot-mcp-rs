@tool
extends RefCounted
## Transaction Handler
## Manages transaction state for grouping multiple operations into a single Undo action

var plugin: EditorPlugin
var _in_transaction: bool = false
var _transaction_name: String = ""
var _transaction_id: String = ""

func _init(p: EditorPlugin) -> void:
	plugin = p

func handle(command: String, params: Dictionary) -> Dictionary:
	match command:
		"begin_transaction":
			return _handle_begin_transaction(params)
		"commit_transaction":
			return _handle_commit_transaction(params)
		"rollback_transaction":
			return _handle_rollback_transaction(params)
		_:
			return {"error": "Unknown transaction command: " + command}

## Check if a transaction is currently active
func is_in_transaction() -> bool:
	return _in_transaction

## Get the UndoRedo manager if in transaction, null otherwise
## Other handlers should check this to avoid creating their own actions
func get_transaction_undo_redo() -> EditorUndoRedoManager:
	if _in_transaction:
		return plugin.get_undo_redo()
	return null

func _handle_begin_transaction(params: Dictionary) -> Dictionary:
	if _in_transaction:
		return {"error": "Transaction already in progress: " + _transaction_name}
	
	var name = params.get("name", "LLM Transaction")
	_transaction_name = name
	_transaction_id = str(Time.get_ticks_msec())
	_in_transaction = true
	
	# Create a single action that will contain all operations
	var ur = plugin.get_undo_redo()
	ur.create_action("LLM: " + name)
	
	return {
		"success": true,
		"transaction_id": _transaction_id,
		"message": "Transaction started: " + name
	}

func _handle_commit_transaction(_params: Dictionary) -> Dictionary:
	if not _in_transaction:
		return {"error": "No transaction in progress"}
	
	# Commit the accumulated action
	var ur = plugin.get_undo_redo()
	ur.commit_action()
	
	var result = {
		"success": true,
		"transaction_id": _transaction_id,
		"message": "Transaction committed: " + _transaction_name
	}
	
	_reset_transaction_state()
	return result

func _handle_rollback_transaction(_params: Dictionary) -> Dictionary:
	if not _in_transaction:
		return {"error": "No transaction in progress"}
	
	# Discard the action without committing
	# Note: EditorUndoRedoManager doesn't have a direct "discard" method,
	# so we commit an empty action and immediately undo it
	var ur = plugin.get_undo_redo()
	
	# Actually, the safest approach is to just not commit the action
	# The action will be discarded when the reference is lost
	# However, this might leave the UndoRedo in an inconsistent state
	# For now, we commit an empty action which effectively does nothing
	ur.commit_action()
	ur.undo()
	
	var result = {
		"success": true,
		"transaction_id": _transaction_id,
		"message": "Transaction rolled back: " + _transaction_name
	}
	
	_reset_transaction_state()
	return result

func _reset_transaction_state() -> void:
	_in_transaction = false
	_transaction_name = ""
	_transaction_id = ""
