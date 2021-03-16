use super::memoized::Memoized;

struct AsyncMemoized<K,V> {
    memoized: Memoized<K,V>
}