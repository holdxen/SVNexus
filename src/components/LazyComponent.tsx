import { useEffect, useState } from 'react'

function LazyComponent({ children, visible }: { children?: React.ReactNode; visible?: boolean }) {
  const [created, setCreated] = useState(visible)
  useEffect(() => {
    if (visible) {
      setCreated(true)
    }
  }, [visible])
  if (created) return <>{children}</>
  return <></>
}

export default LazyComponent
