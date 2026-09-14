import { RadioGroup, Radio, InputNumber, DatePicker } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useEffect, useState } from 'react'

import { Revision } from '@/bindings/Revision'
import { flex, flex_col, flex_1, hidden, gap_y_1 } from '@/styles/Classes'

export interface RevisionSelectProps {
  kinds: RevisionKind[]
  value: Revision
  onChange: (value: Revision) => void
  className?: string
  horizontal?: boolean
  disableLayout?: boolean
}

export type RevisionKind =
  | 'unspecified'
  | 'number'
  | 'date'
  | 'committed'
  | 'previous'
  | 'base'
  | 'working'
  | 'head'

function isEqual(revision: Revision, kind: RevisionKind, time: Date, revisionNumber: number) {
  if (typeof revision === 'string') {
    return revision === kind
  } else if (typeof revision === 'object') {
    if ('number' in revision) {
      return 'number' === kind && revision.number === revisionNumber
    } else if ('date' in revision) {
      return 'date' === kind && time.getTime() === revision.date
    }
  }
  throw new Error('Invalid revision')
}

function kindOfRevisoin(revision: Revision): RevisionKind {
  if (typeof revision === 'string') {
    return revision
  } else if (typeof revision === 'object') {
    if ('number' in revision) {
      return 'number'
    } else if ('date' in revision) {
      return 'date'
    }
  }
  throw new Error('Invalid revision')
}

function revisionNumberOrDefault(revision: Revision): number {
  if (typeof revision === 'object') {
    if ('number' in revision) {
      return revision.number
    }
  }
  return 0
}

function revisionDateOrDefault(revision: Revision): Date {
  if (typeof revision === 'object') {
    if ('date' in revision) {
      return new Date(revision.date)
    }
  }
  return new Date()
}

export default function RevisionSelect(props: RevisionSelectProps) {
  // const head = 'head'
  // const number = 'number'
  // const committed = 'committed'
  // const previous = 'previous'
  // const date = 'date'
  const [revisionKind, setRevisionKind] = useState<RevisionKind>(kindOfRevisoin(props.value))
  const [revisionNumber, setRevisionNumber] = useState<number>(revisionNumberOrDefault(props.value))
  const [time, setTime] = useState<Date>(revisionDateOrDefault(props.value))

  const get = (): Revision => {
    if (revisionKind === 'base') {
      return 'base'
    } else if (revisionKind === 'unspecified') {
      return 'unspecified'
    } else if (revisionKind === 'committed') {
      return 'committed'
    } else if (revisionKind === 'previous') {
      return 'previous'
    } else if (revisionKind === 'working') {
      return 'working'
    } else if (revisionKind === 'head') {
      return 'head'
    } else if (revisionKind === 'number') {
      return { number: revisionNumber }
    } else if (revisionKind === 'date') {
      return { date: time.getTime() }
    } else {
      throw new Error('Unknown kind')
    }
  }

  const set = (revision: Revision) => {
    if (revision === 'base') {
      setRevisionKind('base')
    } else if (revision === 'committed') {
      setRevisionKind('committed')
    } else if (revision === 'head') {
      setRevisionKind('head')
    } else if (revision === 'previous') {
      setRevisionKind('previous')
    } else if (revision === 'unspecified') {
      setRevisionKind('unspecified')
    } else if (revision === 'working') {
      setRevisionKind('working')
    } else if ('number' in revision) {
      setRevisionKind('number')
      setRevisionNumber(revision.number)
    } else if ('date' in revision) {
      setRevisionKind('date')
      setTime(new Date(revision.date))
    }
  }
  useEffect(() => {
    if (isEqual(props.value, revisionKind, time, revisionNumber)) {
      return
    }
    props.onChange(get())
  }, [revisionKind, revisionNumber, time])

  useEffect(() => {
    if (isEqual(props.value, revisionKind, time, revisionNumber)) {
      return
    }
    set(props.value)
  }, [props.value])

  return (
    <div className={cx(flex, !props.horizontal && flex_col, gap_y_1, props.className)}>
      <RadioGroup value={revisionKind} onChange={(value) => setRevisionKind(value.target.value)}>
        {props.kinds.map((item) => {
          return (
            <Radio className={cx(!props.disableLayout && flex_1)} key={item} value={item}>
              {item.charAt(0).toUpperCase() + item.slice(1)}
            </Radio>
          )
        })}
      </RadioGroup>
      <InputNumber
        value={revisionNumber}
        onChange={(v) => {
          if (typeof v === 'number') {
            setRevisionNumber(v)
          }
        }}
        min={0}
        parser={(value) => value.replace(/[^\d]/g, '')}
        formatter={(value) => `${value}`.replace(/[^\d]/g, '')}
        className={cx(revisionKind !== 'number' && hidden)}
      ></InputNumber>
      <DatePicker
        value={time}
        onChange={(value) => {
          if (value instanceof Date) {
            setTime(value)
          }
        }}
        autoAdjustOverflow
        type="dateTime"
        className={cx(revisionKind !== 'date' && hidden)}
      ></DatePicker>
    </div>
  )
}

// import { RadioGroup, Radio, InputNumber, DatePicker } from '@douyinfe/semi-ui'
// import { cx } from '@linaria/core'

// import { flex, flex_col, flex_1, gap_y_3, hidden } from '@/styles/style'
// import { useEffect, useRef, useState } from 'react'
// import { Revision } from '@/bindings/Revision'

// export interface RevisionSelectProps {
//   kinds: RevisionKind[]
//   value: Revision
//   onChange: (value: Revision) => void
// }

// export type RevisionKind = 'unspecified' | 'number' | 'date' | 'committed' | 'previous' | 'base' | 'working' | 'head'

// function revisionToKind(revision: Revision): RevisionKind {
//   if (typeof revision === 'string') return revision
//   if ('number' in revision) return 'number'
//   return 'date'
// }

// function revisionToNumber(revision: Revision): number {
//   return typeof revision === 'object' && 'number' in revision ? revision.number : 0
// }

// function revisionToDate(revision: Revision): Date {
//   return typeof revision === 'object' && 'date' in revision ? new Date(revision.date) : new Date()
// }

// export default function RevisionSelect(props: RevisionSelectProps) {
//   const [revisionKind, setRevisionKind] = useState<RevisionKind>(() => revisionToKind(props.value))
//   const [revisionNumber, setRevisionNumber] = useState(() => revisionToNumber(props.value))
//   const [time, setTime] = useState(() => revisionToDate(props.value))

//   // 同步外部 value 变化到内部 state（但不触发 onChange）
//   const isInternalUpdate = useRef(false)
//   useEffect(() => {
//     if (!isInternalUpdate.current) {
//       setRevisionKind(revisionToKind(props.value))
//       setRevisionNumber(revisionToNumber(props.value))
//       setTime(revisionToDate(props.value))
//     }
//     isInternalUpdate.current = false
//   }, [props.value])

//   const emitChange = (kind: RevisionKind, num: number, date: Date) => {
//     let revision: Revision
//     if (kind === 'number') {
//       revision = { number: num }
//     } else if (kind === 'date') {
//       revision = { date: date.getTime() }
//     } else {
//       revision = kind
//     }
//     isInternalUpdate.current = true
//     props.onChange(revision)
//   }

//   return (
//     <div className={cx(flex, flex_col, flex_1, gap_y_3)}>
//       <RadioGroup
//         value={revisionKind}
//         onChange={(e) => {
//           const kind = e.target.value as RevisionKind
//           setRevisionKind(kind)
//           emitChange(kind, revisionNumber, time)
//         }}
//       >
//         {props.kinds.map((item) => (
//           <Radio className={cx(flex_1)} key={item} value={item}>
//             {item.charAt(0).toUpperCase() + item.slice(1)}
//           </Radio>
//         ))}
//       </RadioGroup>
//       <InputNumber
//         value={revisionNumber}
//         onChange={(v) => {
//           if (typeof v === 'number') {
//             setRevisionNumber(v)
//             emitChange(revisionKind, v, time)
//           }
//         }}
//         min={0}
//         parser={(value) => value.replace(/[^\d]/g, '')}
//         formatter={(value) => `${value}`.replace(/[^\d]/g, '')}
//         className={cx(revisionKind !== 'number' && hidden)}
//       />
//       <DatePicker
//         value={time}
//         onChange={(value) => {
//           if (value instanceof Date) {
//             setTime(value)
//             emitChange(revisionKind, revisionNumber, value)
//           }
//         }}
//         autoAdjustOverflow
//         type="dateTime"
//         className={cx(revisionKind !== 'date' && hidden)}
//       />
//     </div>
//   )
// }
