const COLORS = ['sand', 'moss', 'river', 'clay']
const COLUMNS = 8
const ROWS = 8

const gridElement = document.querySelector('#grid')
const scoreElement = document.querySelector('#score')
const bestElement = document.querySelector('#best')
const leftElement = document.querySelector('#left')
const statusElement = document.querySelector('#status')
const restartButton = document.querySelector('#restart')

const state = {
  cells: [],
  cleared: 0,
  best: Number(localStorage.getItem('pebble-match-best') || 0),
}

function index(column, row) {
  return row * COLUMNS + column
}

function neighbors(position) {
  const column = position % COLUMNS
  const row = Math.floor(position / COLUMNS)
  return [
    column > 0 ? position - 1 : null,
    column < COLUMNS - 1 ? position + 1 : null,
    row > 0 ? position - COLUMNS : null,
    row < ROWS - 1 ? position + COLUMNS : null,
  ].filter((value) => value !== null)
}

function groupAt(position) {
  const color = state.cells[position]
  if (!color) return []
  const found = []
  const pending = [position]
  const seen = new Set(pending)
  while (pending.length > 0) {
    const current = pending.pop()
    found.push(current)
    neighbors(current).forEach((next) => {
      if (!seen.has(next) && state.cells[next] === color) {
        seen.add(next)
        pending.push(next)
      }
    })
  }
  return found
}

function hasMove() {
  return state.cells.some((color, position) => color && groupAt(position).length >= 2)
}

function collapse() {
  for (let column = 0; column < COLUMNS; column += 1) {
    const pebbles = []
    for (let row = ROWS - 1; row >= 0; row -= 1) {
      const color = state.cells[index(column, row)]
      if (color) pebbles.push(color)
    }
    for (let row = ROWS - 1; row >= 0; row -= 1) {
      state.cells[index(column, row)] = pebbles[ROWS - 1 - row] ?? null
    }
  }
}

function render(marked = []) {
  const markedSet = new Set(marked)
  scoreElement.textContent = String(state.cleared)
  bestElement.textContent = String(state.best)
  leftElement.textContent = String(state.cells.filter(Boolean).length)
  gridElement.replaceChildren()
  state.cells.forEach((color, position) => {
    const button = document.createElement('button')
    button.type = 'button'
    button.className = `cell${color ? ` ${color}` : ''}${markedSet.has(position) ? ' marked' : ''}`
    button.setAttribute('aria-label', color ? `${color} pebble` : 'Empty space')
    button.disabled = !color
    button.addEventListener('click', () => play(position))
    gridElement.append(button)
  })
}

function finish(message) {
  state.best = Math.max(state.best, state.cleared)
  localStorage.setItem('pebble-match-best', String(state.best))
  statusElement.textContent = message
  render()
}

function play(position) {
  const group = groupAt(position)
  if (group.length < 2) {
    statusElement.textContent = 'That pebble needs a matching neighbor.'
    render([position])
    return
  }
  group.forEach((item) => { state.cells[item] = null })
  state.cleared += group.length
  collapse()
  if (state.cells.every((color) => color === null)) {
    finish(`Board cleared. ${state.cleared} pebbles removed.`)
    return
  }
  if (!hasMove()) {
    finish(`No matches left. ${state.cleared} pebbles removed.`)
    return
  }
  statusElement.textContent = `Removed ${group.length}. The pebbles above drop down.`
  render()
}

function restart() {
  state.cells = Array.from({ length: COLUMNS * ROWS }, () => (
    COLORS[Math.floor(Math.random() * COLORS.length)]
  ))
  state.cleared = 0
  statusElement.textContent = 'Tap two or more touching pebbles of the same color.'
  render()
}

restartButton.addEventListener('click', restart)
restart()
