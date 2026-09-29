import { setTimeout as sleep } from 'node:timers/promises'
import { test } from 'vitest'

test('c', async () => {
  await sleep(500)
})
