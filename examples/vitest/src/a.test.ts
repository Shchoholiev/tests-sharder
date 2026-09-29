import { setTimeout as sleep } from 'node:timers/promises'
import { test } from 'vitest'

test('a', async () => {
  await sleep(600)
})
