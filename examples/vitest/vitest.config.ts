import { readFileSync } from 'node:fs'
import path from 'node:path'
import { defineConfig } from 'vitest/config'
import { BaseSequencer, type TestSpecification } from 'vitest/node'

type Shard = { tests: Array<{ id: string }> }

class PlannedSequencer extends BaseSequencer {
  override async shard(files: TestSpecification[]) {
    // Vitest calls this only when --shard is set.
    const index = this.ctx.config.shard!.index - 1
    const plan = readFileSync(process.env.SHARD_PLAN!, 'utf8')
      .trim()
      .split('\n')
      .map((line) => JSON.parse(line) as Shard)

    const byPath = new Map(
      files.map((file) => [path.relative(process.cwd(), file.moduleId), file]),
    )
    return plan[index].tests.map(({ id }) => {
      const file = byPath.get(id)
      if (!file) throw new Error(`Unknown test file in Rust plan: ${id}`)
      return file
    })
  }
}

export default defineConfig({
  test: {
    fileParallelism: false,
    reporters: ['verbose'],
    sequence: { sequencer: PlannedSequencer },
  },
})
