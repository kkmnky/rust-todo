import { Button, Card, Checkbox, Grid, Stack, Typography } from '@mui/material'
import { ChangeEventHandler, FC } from 'react'
import { Todo } from '../types/todo'

type Props = {
  todo: Todo
  onUpdate: (todo: Todo) => void
  onDelete: (id: number) => void
}

const TodoItem: FC<Props> = ({ todo, onUpdate, onDelete }) => {
  const handleCompletedCheckbox: ChangeEventHandler = () => {
    onUpdate({
      ...todo,
      completed: !todo.completed,
    })
  }

  const handleDelete = () => onDelete(todo.id)

  return (
    <Card sx={{ p: 1 }}>
      <Grid container spacing={2} alignItems='center'>
        <Grid item xs={1}>
          <Checkbox
            checked={todo.completed}
            onChange={handleCompletedCheckbox}
          />
        </Grid>
        <Grid item xs={9}>
          <Stack spacing={1}>
            <Typography variant='caption' fontSize={16}>
              {todo.text}
            </Typography>
          </Stack>
        </Grid>
        <Grid item xs={1}>
          <Button onClick={handleDelete} color='error'>
            delete
          </Button>
        </Grid>
      </Grid>
    </Card>
  )
}

export default TodoItem
