import { TextArea } from '@douyinfe/semi-ui'
import { TextAreaProps } from '@douyinfe/semi-ui/lib/es/input'

export interface PureTextAreaProps extends Omit<
  TextAreaProps,
  'spellCheck' | 'autoCorrect' | 'autoComplete'
> {}

export default function PureTextArea(props: PureTextAreaProps) {
  return (
    <TextArea {...props} spellCheck={false} autoCorrect={'off'} autoComplete={'off'}></TextArea>
  )
}
