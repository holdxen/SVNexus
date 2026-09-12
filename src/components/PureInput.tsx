import { Input } from '@douyinfe/semi-ui'
import { InputProps } from '@douyinfe/semi-ui/lib/es/input'
import { RefObject } from 'react'

export interface PureInputProps extends Omit<InputProps, 'spellCheck' | 'autoCorrect' | 'autoComplete'> {
  ref?: RefObject<HTMLInputElement | null>
}

export default function PureInput(props: PureInputProps) {
  return <Input {...props} spellCheck={false} autoCorrect={'off'} autoComplete={'off'}></Input>
}
