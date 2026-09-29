import { setTimeout as sleep } from 'node:timers/promises'
import { test } from 'vitest'

test('d', async () => {
  await sleep(400)
})
