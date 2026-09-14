import './styles/fonts.css'
// main.tsx
import ReactDOM from 'react-dom/client'

import App from './App'
import './Style'

import '@styles/hover.css'
import 'overlayscrollbars/overlayscrollbars.css'
import 'react-responsive-modal/styles.css'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(<App />)
