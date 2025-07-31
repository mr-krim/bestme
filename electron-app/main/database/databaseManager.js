const Database = require('better-sqlite3');
const { app } = require('electron');
const path = require('path');
const { v4: uuidv4 } = require('crypto');

class DatabaseManager {
  constructor() {
    this.db = null;
    this.dbPath = path.join(app.getPath('userData'), 'bestme.db');
  }
  
  async initialize() {
    try {
      console.log('Initializing database at:', this.dbPath);
      
      // Ensure directory exists
      const fs = require('fs').promises;
      const dbDir = path.dirname(this.dbPath);
      await fs.mkdir(dbDir, { recursive: true });
      
      // Open database
      this.db = new Database(this.dbPath);
      
      // Enable WAL mode for better concurrency
      this.db.pragma('journal_mode = WAL');
      
      // Create tables
      await this.createTables();
      
      console.log('Database initialized successfully');
    } catch (error) {
      console.error('Failed to initialize database:', error);
      throw error;
    }
  }
  
  async createTables() {
    // Transcripts table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS transcripts (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        word_count INTEGER DEFAULT 0,
        language TEXT,
        model_used TEXT
      )
    `);
    
    // Chat sessions table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS chat_sessions (
        id TEXT PRIMARY KEY,
        title TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        model_used TEXT,
        total_messages INTEGER DEFAULT 0
      )
    `);
    
    // Chat messages table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS chat_messages (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        sender TEXT NOT NULL, -- 'user', 'assistant', 'system'
        content TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        token_count INTEGER,
        FOREIGN KEY (session_id) REFERENCES chat_sessions (id) ON DELETE CASCADE
      )
    `);
    
    // Voice commands history table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS voice_commands (
        id TEXT PRIMARY KEY,
        command_type TEXT NOT NULL,
        trigger_text TEXT NOT NULL,
        executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        success BOOLEAN DEFAULT TRUE,
        result TEXT
      )
    `);
    
    // Vocabulary entries table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS vocabulary (
        id TEXT PRIMARY KEY,
        term TEXT NOT NULL UNIQUE,
        boost_factor REAL DEFAULT 1.0,
        category TEXT,
        variants TEXT, -- JSON array of variants
        context TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        enabled BOOLEAN DEFAULT TRUE,
        usage_count INTEGER DEFAULT 0
      )
    `);
    
    // Create indexes for better performance
    this.db.exec(`
      CREATE INDEX IF NOT EXISTS idx_transcripts_created_at ON transcripts (created_at DESC);
      CREATE INDEX IF NOT EXISTS idx_chat_sessions_created_at ON chat_sessions (created_at DESC);
      CREATE INDEX IF NOT EXISTS idx_chat_messages_session_id ON chat_messages (session_id);
      CREATE INDEX IF NOT EXISTS idx_chat_messages_timestamp ON chat_messages (timestamp);
      CREATE INDEX IF NOT EXISTS idx_voice_commands_executed_at ON voice_commands (executed_at DESC);
      CREATE INDEX IF NOT EXISTS idx_vocabulary_term ON vocabulary (term);
    `);
    
    console.log('Database tables created/verified');
  }
  
  // Transcript operations
  async saveTranscript(title, content, options = {}) {
    const id = uuidv4();
    const wordCount = content.split(/\s+/).length;
    
    const stmt = this.db.prepare(`
      INSERT INTO transcripts (id, title, content, word_count, language, model_used)
      VALUES (?, ?, ?, ?, ?, ?)
    `);
    
    try {
      stmt.run(id, title, content, wordCount, options.language || null, options.model || null);
      return { id, title, content, wordCount };
    } catch (error) {
      console.error('Failed to save transcript:', error);
      throw error;
    }
  }
  
  async getSavedTranscripts(limit = 50) {
    const stmt = this.db.prepare(`
      SELECT id, title, created_at, updated_at, word_count, language
      FROM transcripts
      ORDER BY created_at DESC
      LIMIT ?
    `);
    
    try {
      const rows = stmt.all(limit);
      return rows.map(row => ({
        id: row.id,
        name: row.title,
        subtitle: `${row.word_count} words • ${new Date(row.created_at).toLocaleDateString()}`,
        created_at: row.created_at,
        word_count: row.word_count,
        language: row.language
      }));
    } catch (error) {
      console.error('Failed to get saved transcripts:', error);
      return [];
    }
  }
  
  async getTranscript(id) {
    const stmt = this.db.prepare('SELECT * FROM transcripts WHERE id = ?');
    
    try {
      const row = stmt.get(id);
      if (!row) {
        throw new Error(`Transcript not found: ${id}`);
      }
      
      return {
        id: row.id,
        title: row.title,
        content: row.content,
        created_at: row.created_at,
        updated_at: row.updated_at,
        word_count: row.word_count,
        language: row.language,
        model_used: row.model_used
      };
    } catch (error) {
      console.error('Failed to get transcript:', error);
      throw error;
    }
  }
  
  async deleteTranscript(id) {
    const stmt = this.db.prepare('DELETE FROM transcripts WHERE id = ?');
    
    try {
      const result = stmt.run(id);
      if (result.changes === 0) {
        throw new Error(`Transcript not found: ${id}`);
      }
      return true;
    } catch (error) {
      console.error('Failed to delete transcript:', error);
      throw error;
    }
  }
  
  // Chat operations
  async getChatSessions(limit = 50) {
    const stmt = this.db.prepare(`
      SELECT cs.*, COUNT(cm.id) as message_count
      FROM chat_sessions cs
      LEFT JOIN chat_messages cm ON cs.id = cm.session_id
      GROUP BY cs.id
      ORDER BY cs.updated_at DESC
      LIMIT ?
    `);
    
    try {
      const rows = stmt.all(limit);
      return rows.map(row => ({
        id: row.id,
        name: row.title || `Chat ${new Date(row.created_at).toLocaleDateString()}`,
        subtitle: `${row.message_count} messages • ${row.model_used || 'AI'}`,
        created_at: row.created_at,
        updated_at: row.updated_at,
        message_count: row.message_count
      }));
    } catch (error) {
      console.error('Failed to get chat sessions:', error);
      return [];
    }
  }
  
  async getChatSession(id) {
    // Get session info
    const sessionStmt = this.db.prepare('SELECT * FROM chat_sessions WHERE id = ?');
    const session = sessionStmt.get(id);
    
    if (!session) {
      throw new Error(`Chat session not found: ${id}`);
    }
    
    // Get messages
    const messagesStmt = this.db.prepare(`
      SELECT * FROM chat_messages 
      WHERE session_id = ? 
      ORDER BY timestamp ASC
    `);
    const messages = messagesStmt.all(id);
    
    return {
      id: session.id,
      title: session.title,
      created_at: session.created_at,
      updated_at: session.updated_at,
      model_used: session.model_used,
      messages: messages.map(msg => ({
        id: msg.id,
        sender: msg.sender,
        text: msg.content,
        timestamp: msg.timestamp
      }))
    };
  }
  
  async createChatSession(title = null, model = null) {
    const id = uuidv4();
    const sessionTitle = title || `Chat ${new Date().toLocaleDateString()}`;
    
    const stmt = this.db.prepare(`
      INSERT INTO chat_sessions (id, title, model_used)
      VALUES (?, ?, ?)
    `);
    
    try {
      stmt.run(id, sessionTitle, model);
      return id;
    } catch (error) {
      console.error('Failed to create chat session:', error);
      throw error;
    }
  }
  
  async addChatMessage(sessionId, sender, content, tokenCount = null) {
    const id = uuidv4();
    
    const stmt = this.db.prepare(`
      INSERT INTO chat_messages (id, session_id, sender, content, token_count)
      VALUES (?, ?, ?, ?, ?)
    `);
    
    try {
      stmt.run(id, sessionId, sender, content, tokenCount);
      
      // Update session's updated_at
      const updateStmt = this.db.prepare(`
        UPDATE chat_sessions 
        SET updated_at = CURRENT_TIMESTAMP 
        WHERE id = ?
      `);
      updateStmt.run(sessionId);
      
      return {
        id,
        sender,
        text: content,
        timestamp: new Date().toISOString()
      };
    } catch (error) {
      console.error('Failed to add chat message:', error);
      throw error;
    }
  }
  
  async deleteChatSession(id) {
    // Delete messages first (cascade should handle this, but being explicit)
    const deleteMessagesStmt = this.db.prepare('DELETE FROM chat_messages WHERE session_id = ?');
    const deleteSessionStmt = this.db.prepare('DELETE FROM chat_sessions WHERE id = ?');
    
    try {
      const transaction = this.db.transaction(() => {
        deleteMessagesStmt.run(id);
        const result = deleteSessionStmt.run(id);
        
        if (result.changes === 0) {
          throw new Error(`Chat session not found: ${id}`);
        }
      });
      
      transaction();
      return true;
    } catch (error) {
      console.error('Failed to delete chat session:', error);
      throw error;
    }
  }
  
  // Voice commands operations
  async logVoiceCommand(commandType, triggerText, success = true, result = null) {
    const id = uuidv4();
    
    const stmt = this.db.prepare(`
      INSERT INTO voice_commands (id, command_type, trigger_text, success, result)
      VALUES (?, ?, ?, ?, ?)
    `);
    
    try {
      stmt.run(id, commandType, triggerText, success, result);
    } catch (error) {
      console.error('Failed to log voice command:', error);
    }
  }
  
  async getVoiceCommandHistory(limit = 100) {
    const stmt = this.db.prepare(`
      SELECT * FROM voice_commands
      ORDER BY executed_at DESC
      LIMIT ?
    `);
    
    try {
      return stmt.all(limit);
    } catch (error) {
      console.error('Failed to get voice command history:', error);
      return [];
    }
  }
  
  // Vocabulary operations
  async addVocabularyEntry(term, boostFactor = 1.0, category = null, variants = [], context = null) {
    const id = uuidv4();
    
    const stmt = this.db.prepare(`
      INSERT INTO vocabulary (id, term, boost_factor, category, variants, context)
      VALUES (?, ?, ?, ?, ?, ?)
    `);
    
    try {
      stmt.run(id, term, boostFactor, category, JSON.stringify(variants), context);
      return { id, term, boostFactor, category, variants, context };
    } catch (error) {
      console.error('Failed to add vocabulary entry:', error);
      throw error;
    }
  }
  
  async getVocabularyEntries() {
    const stmt = this.db.prepare(`
      SELECT * FROM vocabulary
      WHERE enabled = TRUE
      ORDER BY term ASC
    `);
    
    try {
      const rows = stmt.all();
      return rows.map(row => ({
        id: row.id,
        term: row.term,
        boost_factor: row.boost_factor,
        category: row.category,
        variants: JSON.parse(row.variants || '[]'),
        context: row.context,
        usage_count: row.usage_count
      }));
    } catch (error) {
      console.error('Failed to get vocabulary entries:', error);
      return [];
    }
  }
  
  // Database maintenance
  async vacuum() {
    try {
      this.db.exec('VACUUM');
      console.log('Database vacuumed successfully');
    } catch (error) {
      console.error('Failed to vacuum database:', error);
    }
  }
  
  async getStats() {
    try {
      const transcriptCount = this.db.prepare('SELECT COUNT(*) as count FROM transcripts').get().count;
      const chatSessionCount = this.db.prepare('SELECT COUNT(*) as count FROM chat_sessions').get().count;
      const chatMessageCount = this.db.prepare('SELECT COUNT(*) as count FROM chat_messages').get().count;
      const vocabularyCount = this.db.prepare('SELECT COUNT(*) as count FROM vocabulary WHERE enabled = TRUE').get().count;
      
      return {
        transcripts: transcriptCount,
        chatSessions: chatSessionCount,
        chatMessages: chatMessageCount,
        vocabularyEntries: vocabularyCount
      };
    } catch (error) {
      console.error('Failed to get database stats:', error);
      return {
        transcripts: 0,
        chatSessions: 0,
        chatMessages: 0,
        vocabularyEntries: 0
      };
    }
  }
  
  close() {
    if (this.db) {
      this.db.close();
      console.log('Database connection closed');
    }
  }
}

module.exports = DatabaseManager;