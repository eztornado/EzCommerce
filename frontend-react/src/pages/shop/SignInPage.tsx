import { useState } from 'react';
import { Container, Title, TextInput, PasswordInput, Button, Stack, Paper, Tabs, Alert } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useNavigate } from 'react-router-dom';
import { api } from '../../api/client';
import { useAuth } from '../../store';

export default function SignInPage() {
  const navigate = useNavigate();
  const login = useAuth((s) => s.login);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  const [error, setError] = useState('');

  const doLogin = async () => {
    setError('');
    try { await login(email, password); navigate('/account'); }
    catch (err: any) { setError(err.message); }
  };

  const doRegister = async () => {
    setError('');
    try {
      await api.post('customer/register', {
        first_name: firstName, last_name: lastName, email,
        password, password_confirmation: password,
      });
      await login(email, password);
      navigate('/account');
    } catch (err: any) { setError(err.message); }
  };

  return (
    <Container size={420} py="xl">
      <Paper withBorder shadow="sm" p="xl" radius="md">
        <Tabs defaultValue="signin">
          <Tabs.List>
            <Tabs.Tab value="signin">Sign in</Tabs.Tab>
            <Tabs.Tab value="register">Create account</Tabs.Tab>
          </Tabs.List>
          {error && <Alert icon={<IconAlertCircle size={16} />} color="red" mt="md">{error}</Alert>}
          <Tabs.Panel value="signin" pt="md">
            <Stack>
              <TextInput label="Email" value={email} onChange={(e) => setEmail(e.currentTarget.value)} />
              <PasswordInput label="Password" value={password} onChange={(e) => setPassword(e.currentTarget.value)} />
              <Button onClick={doLogin}>Sign in</Button>
            </Stack>
          </Tabs.Panel>
          <Tabs.Panel value="register" pt="md">
            <Stack>
              <TextInput label="First name" value={firstName} onChange={(e) => setFirstName(e.currentTarget.value)} />
              <TextInput label="Last name" value={lastName} onChange={(e) => setLastName(e.currentTarget.value)} />
              <TextInput label="Email" value={email} onChange={(e) => setEmail(e.currentTarget.value)} />
              <PasswordInput label="Password" value={password} onChange={(e) => setPassword(e.currentTarget.value)} />
              <Button onClick={doRegister}>Create account</Button>
            </Stack>
          </Tabs.Panel>
        </Tabs>
      </Paper>
    </Container>
  );
}
