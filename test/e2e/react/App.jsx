import { useState } from 'react'

function Comp({ enabled }) {
  return (
    <div>
      {
        enabled ? (
          <a href="https://vite.dev" target="_blank">
            <img src="https://placeholder" className="logo" alt="Vite logo" />
          </a>
        ) : null
      }
    </div>
  )
}

function App() {
  const [count, setCount] = useState(0)

  return (
    <>
    1:
      <Comp enabled={false} />
    2:
      <Comp enabled={0} />
    3:
      <Comp enabled={""} />
    </>
  )
}

export default App
