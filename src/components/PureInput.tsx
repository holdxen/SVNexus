import { Input } from '@douyinfe/semi-ui'
import { InputProps } from '@douyinfe/semi-ui/lib/es/input'
import { forwardRef } from 'react'

export interface PureInputProps extends Omit<
  InputProps,
  'spellCheck' | 'autoCorrect' | 'autoComplete'
> {}

export default forwardRef<HTMLInputElement, PureInputProps>(function PureInput(props, ref) {
  return (
    <Input {...props} ref={ref} spellCheck={false} autoCorrect={'off'} autoComplete={'off'}></Input>
  )
})
