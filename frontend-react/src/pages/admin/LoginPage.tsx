import { useState } from 'react';
import { Container, Title, TextInput, PasswordInput, Button, Stack, Paper, Alert, Center } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useNavigate } from 'react-router-dom';
import { api } from '../../api/client';
import { useAdminAuth } from '../../store';

export default function AdminLoginPage() {
  const navigate = useNavigate();
  const login = useAdminAuth((s) => s.login);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  const doLogin = async () => {
    setError('');
    try {
      await login(email, password);
      navigate('/admin');
    } catch (err: any) { setError(err.message); }
  };

  return (
    <Center mih="100vh">
      <Container size={420}>
        <Paper withBorder shadow="sm" p="xl" radius="md">
          <Title order={2} ta="center" mb="md">EzCommerce Admin</Title>
          {error && <Alert icon={<IconAlertCircle size={16} />} color="red" mb="md">{error}</Alert>}
          <Stack>
            <TextInput label="Email" value={email} onChange={(e) => setEmail(e.currentTarget.value)} />
            <PasswordInput label="Password" value={password} onChange={(e) => setPassword(e.currentTarget.value)} />
            <Button onClick={doLogin}>Sign in</Button>
          </Stack>
        </Paper>
      </Container>
    </Center>
  );
}
