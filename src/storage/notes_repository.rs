/// Repository for notes persistence
use crate::errors::NoteError;
use crate::models::note::Note;
use chrono::{DateTime, Utc};
use rusqlite::Connection;

/// Notes repository for database operations
pub struct NotesRepository;

impl NotesRepository {
    /// Save a note to the database
    ///
    /// Returns the ID of the saved note
    pub fn save(conn: &Connection, note: &Note) -> Result<i64, NoteError> {
        conn.execute(
            "INSERT INTO notes (content, created_at) VALUES (?1, ?2)",
            (&note.content, note.created_at.to_rfc3339()),
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get a note by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Note, NoteError> {
        let mut stmt = conn.prepare("SELECT id, content, created_at FROM notes WHERE id = ?1")?;

        let note = stmt.query_row([id], |row| {
            let created_at_str: String = row.get(2)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            Ok(Note {
                id: Some(row.get(0)?),
                content: row.get(1)?,
                created_at,
            })
        })?;

        Ok(note)
    }

    /// List all notes in chronological order (oldest first)
    pub fn list_all(conn: &Connection) -> Result<Vec<Note>, NoteError> {
        let mut stmt =
            conn.prepare("SELECT id, content, created_at FROM notes ORDER BY created_at ASC")?;

        let notes = stmt
            .query_map([], |row| {
                let created_at_str: String = row.get(2)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(Note {
                    id: Some(row.get(0)?),
                    content: row.get(1)?,
                    created_at,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    /// List notes filtered by entity name (case-insensitive)
    pub fn list_by_entity(conn: &Connection, entity_name: &str) -> Result<Vec<Note>, NoteError> {
        let lowercase_name = entity_name.to_lowercase();

        let mut stmt = conn.prepare(
            "SELECT DISTINCT n.id, n.content, n.created_at
             FROM notes n
             INNER JOIN note_entities ne ON n.id = ne.note_id
             INNER JOIN entities e ON ne.entity_id = e.id
             WHERE e.name = ?1
             ORDER BY n.created_at ASC",
        )?;

        let notes = stmt
            .query_map([lowercase_name], |row| {
                let created_at_str: String = row.get(2)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(Note {
                    id: Some(row.get(0)?),
                    content: row.get(1)?,
                    created_at,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::get_memory_connection;
    use crate::storage::migrations::run_migrations;

    #[test]
    fn test_save_note() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let note = Note::new("Test note".to_string()).unwrap();
        let id = NotesRepository::save(&conn, &note).unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_get_note_by_id() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let note = Note::new("Test note".to_string()).unwrap();
        let id = NotesRepository::save(&conn, &note).unwrap();

        let retrieved = NotesRepository::get_by_id(&conn, id).unwrap();
        assert_eq!(retrieved.content, "Test note");
        assert_eq!(retrieved.id, Some(id));
    }

    #[test]
    fn test_list_all_notes_empty() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let notes = NotesRepository::list_all(&conn).unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn test_list_all_notes() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let note1 = Note::new("First".to_string()).unwrap();
        let note2 = Note::new("Second".to_string()).unwrap();

        NotesRepository::save(&conn, &note1).unwrap();
        NotesRepository::save(&conn, &note2).unwrap();

        let notes = NotesRepository::list_all(&conn).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].content, "First");
        assert_eq!(notes[1].content, "Second");
    }
}
