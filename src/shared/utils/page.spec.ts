import { describe, expect, it } from 'vitest'
import { buildCards } from './page'

describe('buildCards', () => {
  it('returns the same card list', () => {
    const cards = [{ titleKey: 'a', descKey: 'b' }]

    expect(buildCards(cards)).toEqual(cards)
  })
})
