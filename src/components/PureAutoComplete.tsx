import AutoComplete, {
  AutoCompleteItems,
  AutoCompleteProps,
} from '@douyinfe/semi-ui/lib/es/autoComplete'
import { useEffect, useRef } from 'react'

export interface PureAutoCompleteProps<T extends AutoCompleteItems> extends AutoCompleteProps<T> {}

export default function PureAutoComplete<T extends AutoCompleteItems>(
  props: PureAutoCompleteProps<T>,
) {
  const dom = useRef<AutoComplete<T>>(null)
  useEffect(() => {
    if (dom.current === null) {
      return
    }
    const element = dom.current.triggerRef?.current.querySelector('.semi-input')
    if (element) {
      const input = element as HTMLInputElement
      input.spellcheck = false
      input.autocorrect = false
      input.autocomplete = 'off'
    }
  }, [])
  return (
    <AutoComplete
      {...props}
      ref={dom}
      spellCheck={false}
      autoCorrect={'off'}
      autoComplete={'off'}
    ></AutoComplete>
  )
}
