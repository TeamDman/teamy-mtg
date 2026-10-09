# Command completion budgets

Each successful leaf returns an explicit execution-only budget. Dispatch preserves it; top-level timing includes output and checks even output errors. Only strictly exceeding a budget warns. DEBUG dispatch/completion and command-span tracing remain independent; serialized values are unchanged.

- Home/cache show: 1 second for path inspection; detached open: 3 seconds.
- Cache clean: 3 seconds + 20 ms per removed owned cache entry.
- Card list/search: 3 seconds + 10 ms per requested result limit, including local query and serialization work.
- Deck list/card list: 3 seconds + 20 ms per returned entry. Deck create: 3 seconds for one new metadata file. Deck card add/remove: 3 seconds + 20 ms per resulting deck entry for metadata validation and rendering.
- Image list: 3 seconds + 200 ms per resolved card face for cached-image validation.
- Image sync: 3 seconds + 601 seconds per resolved face. Each finite request already has a 600-second timeout, plus pacing and cache validation. Cached faces conservatively retain this allowance.
- PDF generate: 3 seconds + 1 second per quantity-expanded card face offline, or 602 seconds per quantity-expanded face online, covering decode/render plus each finite request timeout. This conservatively includes faces excluded by the selected face option and repeated images.
- DB import: 3 seconds + 2 ms per imported row. Online update adds 1200 seconds for the two finite HTTP requests (600-second timeout each).

No successful command is open-ended, so none are exempted. Formulas use work quantities, never actual elapsed time; duration multiplication saturates. Tests use synthetic fixtures and no external requests.