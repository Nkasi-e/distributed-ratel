-- Atomic token-bucket allow for Redis (KEYS[1] = hash key, ARGV = capacity, refill_per_sec, cost, ttl_ms).
-- Loaded at compile time via include_str! in redis_limiter.rs.

local capacity = tonumber(ARGV[1])
local refill_per_sec = tonumber(ARGV[2])
local cost = tonumber(ARGV[3])
local ttl_ms = tonumber(ARGV[4])

if cost <= 0 then
  return redis.error_reply("invalid_cost")
end

local t = redis.call("TIME")
local now_ms = t[1] * 1000 + math.floor(t[2] / 1000)

local vals = redis.call("HMGET", KEYS[1], "tokens", "ts")
local tokens = tonumber(vals[1])
local ts = tonumber(vals[2])

if tokens == nil then
  tokens = capacity
  ts = now_ms
end

local elapsed_ms = math.max(0, now_ms - ts)
tokens = math.min(capacity, tokens + (elapsed_ms / 1000.0) * refill_per_sec)

local allowed = 0
if tokens >= cost then
  tokens = tokens - cost
  allowed = 1
end

redis.call("HMSET", KEYS[1], "tokens", tokens, "ts", now_ms)
redis.call("PEXPIRE", KEYS[1], ttl_ms)

return allowed
