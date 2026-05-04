-- Sliding-window via ZSET + cardinality (KEYs[1] = zset, KEYS[2]=seq).
-- ARGV: window_ms, max_cost_in_window, request_cost, ttl_ms.


local window_ms = tonumber(ARGV[1])
local max_cost = tonumber(ARGV[2])
local cost = tonumber(ARGV[3])
local ttl_ms = tonumber(ARGV[4])

if cost <= 0 then
    return redis.error_reply("invalid_cost")
end

local t = redis.call("TIME")
local now_ms = t[1] * 1000 + math.floor(t[2] / 1000)

redis.call("ZREMRANGEBYSCORE", KEYS[1], "-inf", now_ms - window_ms)

local current = redis.call("ZCARD", KEYS[1])

if current + cost > max_cost then
    redis.call("PEXPIRE", KEYS[1], ttl_ms)
    redis.call("PEXPIRE", KEYS[2], ttl_ms)
    return 0
end

for _ = 1, cost do
    local id = redis.call("INCR", KEYS[2])
    redis.call("ZADD", KEYS[1], now_ms + id / 1e15, tostring(id))
end

redis.call("PEXPIRE", KEYS[1], ttl_ms)
redis.call("PEXPIRE", KEYS[2], ttl_ms)

return 1