import { useAppStore } from './stores/app-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import './index.css'

export default function App() {
  const is_setup = useAppStore((s) => s.is_setup)

  if (!is_setup) {
    return <SetupWizard />
  }

  return <AppLayout />
}
