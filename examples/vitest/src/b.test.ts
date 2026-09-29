import { setTimeout as sleep } from 'node:timers/promises'
import { test } from 'vitest'

test('b', async () => {
  await sleep(500)
})
