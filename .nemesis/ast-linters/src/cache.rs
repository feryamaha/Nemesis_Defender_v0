/// Cache LRU para árvores AST parseadas.
///
/// Evita re-parse do mesmo arquivo durante a mesma sessão de hook.
/// A chave é composta por (caminho_do_arquivo, hash_do_conteúdo).
/// Quando o conteúdo muda, o hash muda, e o cache é invalidado automaticamente.

use crate::parser::ParsedTree;
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// Capacidade máxima do cache (32 entradas).
const CACHE_CAPACITY: usize = 32;

/// Chave do cache: (caminho, hash_do_conteúdo).
#[derive(Clone, Debug, PartialEq, Eq)]
struct CacheKey {
    path: String,
    content_hash: u64,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.content_hash.hash(state);
    }
}

/// Valor do cache: árvore parseada e conteúdo anterior.
#[derive(Clone)]
struct CacheValue {
    tree: Option<ParsedTree>, // None para compatibilidade com API antiga
    content: String, // Armazena o conteúdo anterior para calcular diff
}

/// Calcula um hash simples do conteúdo (usando std::collections::hash_map).
fn content_hash(content: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Cache global protegido por mutex.
static CACHE: std::sync::OnceLock<Mutex<LruCache<CacheKey, CacheValue>>> = std::sync::OnceLock::new();

fn cache() -> &'static Mutex<LruCache<CacheKey, CacheValue>> {
    CACHE.get_or_init(|| {
        Mutex::new(LruCache::new(NonZeroUsize::new(CACHE_CAPACITY).unwrap()))
    })
}

/// Verifica se o conteúdo está no cache.
/// Retorna `true` se o cache contém esta chave, `false` caso contrário.
pub fn has(path: &str, content: &str) -> bool {
    let key = CacheKey {
        path: path.to_string(),
        content_hash: content_hash(content),
    };
    if let Ok(cache) = cache().lock() {
        cache.contains(&key)
    } else {
        false // graceful degradation
    }
}

/// Retorna a árvore parseada do cache, se existir.
pub fn get_tree(path: &str, content: &str) -> Option<ParsedTree> {
    let key = CacheKey {
        path: path.to_string(),
        content_hash: content_hash(content),
    };
    if let Ok(mut cache) = cache().lock() {
        cache.get(&key).and_then(|v| v.tree.clone())
    } else {
        None
    }
}

/// Retorna o conteúdo anterior do cache, se existir.
pub fn get_previous_content(path: &str, content: &str) -> Option<String> {
    let key = CacheKey {
        path: path.to_string(),
        content_hash: content_hash(content),
    };
    if let Ok(mut cache) = cache().lock() {
        cache.get(&key).map(|v| v.content.clone())
    } else {
        None
    }
}

/// Insere ou atualiza uma entrada no cache com a árvore parseada.
pub fn put_tree(path: &str, content: &str, tree: ParsedTree) {
    let key = CacheKey {
        path: path.to_string(),
        content_hash: content_hash(content),
    };
    let value = CacheValue {
        tree: Some(tree),
        content: content.to_string(),
    };
    if let Ok(mut cache) = cache().lock() {
        cache.put(key, value);
    }
}

/// Insere ou atualiza uma entrada no cache (compatibilidade com API antiga).
///
/// Nota: Esta função não armazena a árvore parseada. Para parsing incremental,
/// use put_tree() em vez disso.
pub fn put(path: &str, content: &str) {
    let key = CacheKey {
        path: path.to_string(),
        content_hash: content_hash(content),
    };
    let value = CacheValue {
        tree: None, // API antiga não armazena árvore
        content: content.to_string(),
    };
    if let Ok(mut cache) = cache().lock() {
        cache.put(key, value);
    }
}

/// Remove uma entrada do cache pelo caminho (ignora hash).
/// Útil para forçar re-parse em caso de necessidade.
pub fn invalidate(_path: &str) {
    if let Ok(mut cache) = cache().lock() {
        // LRU cache não tem remove_by_key — limpamos tudo para este path
        // (abordagem conservadora: limpa o cache inteiro)
        cache.clear();
    }
}

/// Limpa todo o cache.
pub fn clear() {
    if let Ok(mut cache) = cache().lock() {
        cache.clear();
    }
}

/// Retorna o número de entradas no cache.
pub fn len() -> usize {
    if let Ok(cache) = cache().lock() {
        cache.len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_cache_hit() {
        clear();
        let path = "src/test.ts";
        let content = "const x = 1;";
        assert!(!has(path, content));
        put(path, content);
        assert!(has(path, content));
    }

    #[test]
    #[serial]
    fn test_cache_miss_different_content() {
        clear();
        let path = "src/test.ts";
        put(path, "const x = 1;");
        assert!(!has(path, "const y = 2;"));
    }

    #[test]
    #[serial]
    fn test_cache_miss_different_path() {
        clear();
        put("src/a.ts", "const x = 1;");
        assert!(!has("src/b.ts", "const x = 1;"));
    }

    #[test]
    #[serial]
    fn test_clear() {
        clear();
        put("src/test.ts", "content");
        assert_eq!(len(), 1);
        clear();
        assert_eq!(len(), 0);
    }
}
